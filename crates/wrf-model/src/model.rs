use wrf_compute::{CpuBackend, FieldStorage, GridShape};
use wrf_dynamics::{
    AcousticStepFinalizationKernels, AcousticStepFinalizationMapFactors,
    AcousticStepFinalizationMasses, AcousticStepFinalizationSavedState,
    AcousticStepFinalizationState, AcousticTrajectoryDiagnostics, AcousticTrajectoryInputs,
    AcousticTrajectoryKernels, AcousticTrajectoryMapFactors, AcousticTrajectoryMassInputs,
    AcousticTrajectoryMoistureCoefficients, AcousticTrajectoryPressureInputs,
    AcousticTrajectorySavedState, AcousticTrajectoryTendencies, AcousticTrajectoryTimeLevels,
    AcousticTrajectoryWorkspace, ColumnMassStaggeringKernels, ColumnMassStaggeringPeriodicity,
    DryTendencyAssemblyForwardTendencies, DryTendencyAssemblyKernels,
    DryTendencyAssemblyMapFactors, DryTendencyAssemblyPhase,
    DryTendencyAssemblyRungeKuttaTendencies, DryTendencyAssemblySavedTendencies,
    DryTendencyAssemblyThermodynamics, MoistureSpecies as DynamicsMoistureSpecies,
    OmegaDiagnosisGridMetrics, RungeKuttaPreparationDiagnosticOutputs, RungeKuttaPreparationInputs,
    RungeKuttaPreparationKernels, RungeKuttaPreparationMapFactors, RungeKuttaPreparationMassInputs,
    RungeKuttaPreparationMassOutputs, RungeKuttaPreparationMomentumOutputs,
    RungeKuttaPreparationOutputs, RungeKuttaPreparationThermodynamicInputs,
    RungeKuttaPreparationVelocities,
};
use wrf_physics::{ArwMicrophysicsStage, ArwMicrophysicsState, ArwMicrophysicsTrajectory};

use crate::{
    ArwColumnField, ArwGeopotentialField, ArwMassField, ArwModelCoefficients, ArwModelControls,
    ArwModelError, ArwModelGeometry, ArwModelResult, ArwModelStage, ArwModelStageView,
    ArwModelState, ArwModelWorkspace, ArwRegistryBinding, ArwRegistryField, ArwRestartVolumeField,
};

/// Registry-bound owner of one accepted-stage ARW/Kessler trajectory.
pub struct RegistryBoundArwModel {
    binding: ArwRegistryBinding,
    geometry: ArwModelGeometry,
    coefficients: ArwModelCoefficients,
    controls: ArwModelControls,
    microphysics: ArwMicrophysicsTrajectory,
    geopotential_shape: GridShape,
}

impl RegistryBoundArwModel {
    /// Binds the selected Registry state to the accepted dynamics and Kessler stages.
    ///
    /// # Errors
    ///
    /// Returns a typed Kessler binding or W-level shape-overflow error.
    pub fn try_kessler(
        binding: ArwRegistryBinding,
        geometry: ArwModelGeometry,
        coefficients: ArwModelCoefficients,
        controls: ArwModelControls,
    ) -> ArwModelResult<Self> {
        controls.validate_accepted_projection()?;
        let shape = geometry.shape();
        if coefficients.bottom_top_points() != shape.bottom_top_points() {
            return Err(ArwModelError::CoefficientLengthMismatch {
                name: "model geometry",
                expected: shape.bottom_top_points(),
                actual: coefficients.bottom_top_points(),
            });
        }
        let geopotential_shape = shape;
        let microphysics = ArwMicrophysicsTrajectory::try_kessler(
            geometry.microphysics_domain.clone(),
            binding.moisture_layout(),
            controls.microphysics,
        )?;
        Ok(Self {
            binding,
            geometry,
            coefficients,
            controls,
            microphysics,
            geopotential_shape,
        })
    }

    /// Allocates restart-owned state once on the supplied backend.
    pub fn create_state(&self, backend: &CpuBackend) -> ArwModelResult<ArwModelState> {
        ArwModelState::try_new(backend, &self.binding, self.geometry.shape())
    }

    /// Allocates all diagnostics, tendencies, adapters, and scheme scratch once.
    pub fn create_workspace(&self, backend: &CpuBackend) -> ArwModelResult<ArwModelWorkspace> {
        ArwModelWorkspace::try_new(
            backend,
            self.geometry.shape(),
            self.geometry.microphysics_shape(),
            self.binding.moisture_layout().members().len(),
            &self.microphysics,
        )
    }

    /// Executes the complete accepted-stage projection without instrumentation.
    pub fn advance_short_trajectory(
        &self,
        backend: &CpuBackend,
        state: &mut ArwModelState,
        workspace: &mut ArwModelWorkspace,
    ) -> ArwModelResult<()> {
        self.advance_short_trajectory_with_observer(backend, state, workspace, |_, _| {})
    }

    /// Executes the projection and observes zero-copy stage boundaries.
    ///
    /// This is not a full `solve_em` timestep. It preserves the pinned order of
    /// the included routines while omitting unported tendency, transport, halo,
    /// and additional physics work.
    ///
    /// # Errors
    ///
    /// All state, workspace shape, role-count, and worker contracts are checked
    /// before the W-level adapter or any numerical field changes.
    pub fn advance_short_trajectory_with_observer<Observer>(
        &self,
        backend: &CpuBackend,
        state: &mut ArwModelState,
        workspace: &mut ArwModelWorkspace,
        mut observer: Observer,
    ) -> ArwModelResult<()>
    where
        Observer: FnMut(ArwModelStage, ArwModelStageView<'_>),
    {
        self.preflight(backend, state, workspace)?;
        copy_geopotential_to_dynamics(state, workspace)?;

        self.prepare_runge_kutta(backend, state, workspace)?;
        observer(
            ArwModelStage::RungeKuttaPrepared,
            ArwModelStageView::Dynamics { state, workspace },
        );
        self.assemble_dry_tendencies(backend, state, workspace)?;
        observer(
            ArwModelStage::DryTendenciesAssembled,
            ArwModelStageView::Dynamics { state, workspace },
        );
        self.advance_acoustic(backend, state, workspace)?;
        observer(
            ArwModelStage::AcousticAdvanced,
            ArwModelStageView::Dynamics { state, workspace },
        );
        self.finalize_acoustic(backend, state, workspace)?;
        copy_geopotential_to_registry(state, workspace)?;
        observer(
            ArwModelStage::AcousticFinalized,
            ArwModelStageView::Dynamics { state, workspace },
        );

        self.apply_microphysics(backend, state, workspace, |stage, view| {
            observer(stage, ArwModelStageView::Microphysics(view));
        })
    }

    fn preflight(
        &self,
        backend: &CpuBackend,
        state: &ArwModelState,
        workspace: &ArwModelWorkspace,
    ) -> ArwModelResult<()> {
        workspace.validate_role_counts()?;
        let expected = self.geometry.shape();
        for field in ArwMassField::ALL {
            let actual = state.mass_fields[field as usize].shape();
            if actual != expected {
                return Err(ArwModelError::FieldShapeMismatch {
                    field: ArwRegistryField::Mass(field),
                    expected,
                    actual,
                });
            }
        }
        for field in ArwGeopotentialField::ALL {
            let actual = state.geopotential_fields[field as usize].shape();
            if actual != self.geopotential_shape {
                return Err(ArwModelError::FieldShapeMismatch {
                    field: ArwRegistryField::Geopotential(field),
                    expected: self.geopotential_shape,
                    actual,
                });
            }
        }
        for field in ArwRestartVolumeField::ALL {
            let actual = state.restart_volume_fields[field as usize].shape();
            if actual != expected {
                return Err(ArwModelError::FieldShapeMismatch {
                    field: ArwRegistryField::RestartVolume(field),
                    expected,
                    actual,
                });
            }
        }
        let horizontal = expected.horizontal_shape();
        for field in ArwColumnField::ALL {
            let actual = state.column_fields[field as usize].shape();
            if actual != horizontal {
                return Err(ArwModelError::FieldShapeMismatch {
                    field: ArwRegistryField::Column(field),
                    expected: horizontal,
                    actual,
                });
            }
        }
        for field in crate::ArwMapField::ALL {
            let actual = state.map_fields[field as usize].shape();
            if actual != horizontal {
                return Err(ArwModelError::FieldShapeMismatch {
                    field: ArwRegistryField::Map(field),
                    expected: horizontal,
                    actual,
                });
            }
        }
        if state.moisture_fields.len() != self.binding.moisture_layout().members().len() {
            return Err(ArwModelError::MoistureLayoutCount {
                actual: state.moisture_fields.len(),
            });
        }
        for field in &state.moisture_fields {
            if field.shape() != expected {
                return Err(ArwModelError::InvalidGeometry {
                    component: "Registry moisture field",
                });
            }
        }
        if workspace.shape() != expected {
            return Err(ArwModelError::InvalidGeometry {
                component: "model workspace",
            });
        }
        if workspace.microphysics_shape() != self.geometry.microphysics_shape() {
            return Err(ArwModelError::InvalidGeometry {
                component: "microphysics adapter workspace",
            });
        }
        let actual_workers = backend.worker_count();
        if workspace.worker_count != actual_workers {
            return Err(ArwModelError::WorkspaceWorkerCountMismatch {
                expected: workspace.worker_count,
                actual: actual_workers,
            });
        }
        Ok(())
    }

    fn prepare_runge_kutta(
        &self,
        backend: &CpuBackend,
        state: &mut ArwModelState,
        workspace: &mut ArwModelWorkspace,
    ) -> ArwModelResult<()> {
        let [_u1, u2, _v1, v2, _w1, w2, _t1, _t2, _p, al, _pb, alb] = &mut state.mass_fields;
        let [_mu1, mu2, mub, _rainnc, _rainncv] = &mut state.column_fields;
        let [
            ww,
            _ww_m,
            php,
            _h_diabatic,
            _qv_diabatic,
            _qc_diabatic,
            _rho,
            _th_phy_m_t0,
        ] = &mut state.restart_volume_fields;
        let [
            _ph1,
            ph2,
            phb,
            ru,
            rv,
            rw,
            cqu,
            cqv,
            cqw,
            alt,
            _us,
            _vs,
            _ws,
            _ts,
            _phs,
            _ww1,
            _c2a,
            _pm1,
            _a,
            _alpha,
            _gamma,
            _t2save,
            _ru_m,
            _rv_m,
            _ru_tend,
            _rv_tend,
            _rw_tend,
            _ph_tend,
            _t_tend,
            _ruf,
            _rvf,
            _rwf,
            _phf,
            _tf,
            _geopotential_rhs,
        ] = workspace.volume_fields.as_mut_slice()
        else {
            return Err(role_count("volume workspace"));
        };
        let [
            mut_full,
            muu,
            muv,
            _mu_tend,
            _muf,
            _muus,
            _muvs,
            _muts,
            _mudf,
            _muave,
            _mus,
        ] = workspace.column_fields.as_mut_slice()
        else {
            return Err(role_count("column workspace"));
        };
        let [_msfux, msfuy, _msfvx, msfvx_inv, _msfvy, msftx, msfty, _ht] = &state.map_fields;
        backend.prepare_runge_kutta_step(
            RungeKuttaPreparationOutputs::new(
                RungeKuttaPreparationMassOutputs::new(mut_full, muu, muv),
                RungeKuttaPreparationMomentumOutputs::new(ru, rv, rw),
                RungeKuttaPreparationDiagnosticOutputs::new(ww, cqu, cqv, cqw, alt, php),
            ),
            RungeKuttaPreparationInputs::new(
                RungeKuttaPreparationMassInputs::new(mu2, mub),
                RungeKuttaPreparationVelocities::new(u2, v2, w2),
                RungeKuttaPreparationMapFactors::new(msftx, msfty, msfuy, msfvx_inv),
                self.coefficients.runge_kutta(),
                DynamicsMoistureSpecies::new(&state.moisture_fields),
                RungeKuttaPreparationThermodynamicInputs::new(al, alb, ph2, phb),
                OmegaDiagnosisGridMetrics::new(
                    self.controls.inverse_west_east_grid_spacing(),
                    self.controls.inverse_south_north_grid_spacing(),
                ),
            ),
            &self.geometry.runge_kutta,
            ColumnMassStaggeringPeriodicity::None,
        )?;
        Ok(())
    }

    fn assemble_dry_tendencies(
        &self,
        backend: &CpuBackend,
        state: &ArwModelState,
        workspace: &mut ArwModelWorkspace,
    ) -> ArwModelResult<()> {
        let [
            _ph1,
            _ph2,
            _phb,
            _ru,
            _rv,
            _rw,
            _cqu,
            _cqv,
            _cqw,
            _alt,
            us,
            vs,
            ws,
            ts,
            phs,
            _ww1,
            _c2a,
            _pm1,
            _a,
            _alpha,
            _gamma,
            _t2save,
            _ru_m,
            _rv_m,
            ru_tend,
            rv_tend,
            rw_tend,
            ph_tend,
            t_tend,
            ruf,
            rvf,
            rwf,
            phf,
            tf,
            _geopotential_rhs,
        ] = workspace.volume_fields.as_mut_slice()
        else {
            return Err(role_count("volume workspace"));
        };
        let [
            mut_full,
            _muu,
            _muv,
            mu_tend,
            muf,
            _muus,
            _muvs,
            _muts,
            _mudf,
            _muave,
            _mus,
        ] = workspace.column_fields.as_mut_slice()
        else {
            return Err(role_count("column workspace"));
        };
        let h_diabatic =
            &state.restart_volume_fields[ArwRestartVolumeField::DiabaticHeating as usize];
        let [_msfux, msfuy, msfvx, msfvx_inv, _msfvy, _msftx, msfty, _ht] = &state.map_fields;
        backend.assemble_dry_tendencies(
            DryTendencyAssemblyRungeKuttaTendencies::new(
                ru_tend, rv_tend, rw_tend, ph_tend, t_tend, mu_tend,
            ),
            DryTendencyAssemblyForwardTendencies::new(ruf, rvf, rwf, phf, tf, muf),
            DryTendencyAssemblySavedTendencies::new(us, vs, ws, phs, ts),
            DryTendencyAssemblyThermodynamics::new(h_diabatic, mut_full),
            DryTendencyAssemblyMapFactors::new(msfuy, msfvx, msfvx_inv, msfty),
            self.coefficients.dry_tendency(),
            DryTendencyAssemblyPhase::FirstSubstep,
            &self.geometry.dry_tendency,
        )?;
        Ok(())
    }

    fn advance_acoustic(
        &self,
        backend: &CpuBackend,
        state: &mut ArwModelState,
        workspace: &mut ArwModelWorkspace,
    ) -> ArwModelResult<()> {
        let [u1, u2, v1, v2, w1, w2, t1, t2, p, al, pb, _alb] = &mut state.mass_fields;
        let [mu1, mu2, mub, _rainnc, _rainncv] = &mut state.column_fields;
        let [
            ww,
            ww_m,
            php,
            _h_diabatic,
            _qv_diabatic,
            _qc_diabatic,
            _rho,
            _th_phy_m_t0,
        ] = &mut state.restart_volume_fields;
        let [
            ph1,
            ph2,
            phb,
            _ru,
            _rv,
            _rw,
            cqu,
            cqv,
            cqw,
            alt,
            us,
            vs,
            ws,
            ts,
            phs,
            ww1,
            c2a,
            pm1,
            a,
            alpha,
            gamma,
            t2save,
            ru_m,
            rv_m,
            ru_tend,
            rv_tend,
            rw_tend,
            ph_tend,
            t_tend,
            _ruf,
            _rvf,
            _rwf,
            _phf,
            _tf,
            geopotential_rhs,
        ] = workspace.volume_fields.as_mut_slice()
        else {
            return Err(role_count("volume workspace"));
        };
        let [
            mut_full,
            muu,
            muv,
            mu_tend,
            _muf,
            muus,
            muvs,
            muts,
            mudf,
            muave,
            mus,
        ] = workspace.column_fields.as_mut_slice()
        else {
            return Err(role_count("column workspace"));
        };
        let [msfux, msfuy, msfvx, msfvx_inv, msfvy, msftx, msfty, ht] = &state.map_fields;
        backend.advance_acoustic_trajectory(
            AcousticTrajectoryTimeLevels::new(u1, u2, v1, v2, w1, w2, t1, t2, ph1, ph2, mu1, mu2),
            AcousticTrajectorySavedState::new(us, vs, ws, ts, phs, mus, ww1, c2a),
            AcousticTrajectoryDiagnostics::new(
                ww, muus, muvs, muts, mudf, al, p, pm1, a, alpha, gamma, muave, t2save, ru_m, rv_m,
                ww_m,
            ),
            AcousticTrajectoryWorkspace::new(geopotential_rhs),
            AcousticTrajectoryInputs::new(
                AcousticTrajectoryMassInputs::new(mub, muu, muv, mut_full, mu_tend),
                AcousticTrajectoryPressureInputs::new(pb, alt, php, phb),
                AcousticTrajectoryTendencies::new(ru_tend, rv_tend, rw_tend, t_tend, ph_tend),
                AcousticTrajectoryMoistureCoefficients::new(cqu, cqv, cqw),
                AcousticTrajectoryMapFactors::new(
                    msfux, msfuy, msfvx, msfvx_inv, msfvy, msftx, msfty, ht,
                ),
            ),
            self.coefficients.acoustic(),
            self.controls.acoustic,
            self.geometry.acoustic_regions(),
        )?;
        Ok(())
    }

    fn finalize_acoustic(
        &self,
        backend: &CpuBackend,
        state: &mut ArwModelState,
        workspace: &mut ArwModelWorkspace,
    ) -> ArwModelResult<()> {
        {
            let [
                _mut_full,
                _muu,
                _muv,
                _mu_tend,
                _muf,
                muus,
                muvs,
                muts,
                _mudf,
                _muave,
                _mus,
            ] = workspace.column_fields.as_mut_slice()
            else {
                return Err(role_count("column workspace"));
            };
            backend.stagger_full_column_mass_for_big_step(
                muts,
                muus,
                muvs,
                &self.geometry.final_column_mass,
                ColumnMassStaggeringPeriodicity::None,
            )?;
        }

        let [_u1, u2, _v1, v2, _w1, w2, _t1, t2, _p, _al, _pb, _alb] = &mut state.mass_fields;
        let [_mu1, mu2, _mub, _rainnc, _rainncv] = &mut state.column_fields;
        let [
            ww,
            _ww_m,
            _php,
            h_diabatic,
            _qv_diabatic,
            _qc_diabatic,
            _rho,
            _th_phy_m_t0,
        ] = &mut state.restart_volume_fields;
        let [
            _ph1,
            ph2,
            _phb,
            _ru,
            _rv,
            _rw,
            _cqu,
            _cqv,
            _cqw,
            _alt,
            us,
            vs,
            ws,
            ts,
            phs,
            ww1,
            _c2a,
            _pm1,
            _a,
            _alpha,
            _gamma,
            _t2save,
            _ru_m,
            _rv_m,
            _ru_tend,
            _rv_tend,
            _rw_tend,
            _ph_tend,
            _t_tend,
            _ruf,
            _rvf,
            _rwf,
            _phf,
            _tf,
            _geopotential_rhs,
        ] = workspace.volume_fields.as_mut_slice()
        else {
            return Err(role_count("volume workspace"));
        };
        let [
            mut_full,
            muu,
            muv,
            _mu_tend,
            _muf,
            muus,
            muvs,
            muts,
            _mudf,
            _muave,
            mus,
        ] = workspace.column_fields.as_mut_slice()
        else {
            return Err(role_count("column workspace"));
        };
        let [_msfux, msfuy, msfvx, _msfvx_inv, _msfvy, _msftx, msfty, _ht] = &state.map_fields;
        backend.finalize_acoustic_step(
            AcousticStepFinalizationState::new(u2, v2, w2, t2, ph2, ww, mu2),
            AcousticStepFinalizationMasses::new(mut_full, muts, muu, muus, muv, muvs),
            AcousticStepFinalizationSavedState::new(us, vs, ws, ts, phs, mus, ww1, h_diabatic),
            AcousticStepFinalizationMapFactors::new(msfuy, msfvx, msfty),
            self.coefficients.finalization(),
            self.controls.finalization,
            &self.geometry.finalization,
        )?;
        Ok(())
    }

    fn apply_microphysics<Observer>(
        &self,
        backend: &CpuBackend,
        state: &mut ArwModelState,
        workspace: &mut ArwModelWorkspace,
        mut observer: Observer,
    ) -> ArwModelResult<()>
    where
        Observer: FnMut(ArwModelStage, wrf_physics::ArwMicrophysicsStageView<'_>),
    {
        copy_state_to_microphysics_adapter(state, workspace);
        let mass_shape = state.mass_shape();
        let microphysics_shape = workspace.microphysics_shape();
        let [_u1, _u2, _v1, _v2, _w1, _w2, _t1, t2, _p, _al, _pb, _alb] = &mut state.mass_fields;
        let [_mu1, _mu2, _mub, rainnc, rainncv] = &mut state.column_fields;
        let [
            _ww,
            _ww_m,
            _php,
            h_diabatic,
            qv_diabatic,
            qc_diabatic,
            rho,
            th_phy_m_t0,
        ] = &mut state.restart_volume_fields;
        let [adapter_t2, adapter_al, adapter_alb, adapter_p, adapter_pb] =
            &mut workspace.microphysics_mass_fields;
        let [adapter_ph2, adapter_phb] = &mut workspace.microphysics_geopotential_fields;
        let active_west_east = self.geometry.mass_west_east();
        let active_south_north = self.geometry.mass_south_north();
        self.microphysics.apply_step_with_observer(
            backend,
            ArwMicrophysicsState::new(
                adapter_t2,
                &mut workspace.microphysics_moisture_fields,
                adapter_al,
                adapter_alb,
                adapter_p,
                adapter_pb,
                adapter_ph2,
                adapter_phb,
                rainnc,
                rainncv,
            ),
            &self.geometry.microphysics_tiles,
            &mut workspace.microphysics,
            |stage, view| {
                if stage == ArwMicrophysicsStage::Finished {
                    for (source, destination) in [
                        (view.h_diabatic().values(), h_diabatic.values_mut()),
                        (view.qv_diabatic().values(), qv_diabatic.values_mut()),
                        (view.qc_diabatic().values(), qc_diabatic.values_mut()),
                        (view.dry_air_density().values(), rho.values_mut()),
                    ] {
                        copy_compact_active_to_padded(
                            microphysics_shape,
                            mass_shape,
                            source,
                            destination,
                            active_west_east.clone(),
                            active_south_north.clone(),
                        );
                    }
                    copy_compact_active_to_padded(
                        microphysics_shape,
                        mass_shape,
                        view.perturbation_dry_potential_temperature().values(),
                        th_phy_m_t0.values_mut(),
                        active_west_east.clone(),
                        active_south_north.clone(),
                    );
                }
                let model_stage = match stage {
                    ArwMicrophysicsStage::Prepared => ArwModelStage::MicrophysicsPrepared,
                    ArwMicrophysicsStage::MicrophysicsApplied => ArwModelStage::MicrophysicsApplied,
                    ArwMicrophysicsStage::Finished => ArwModelStage::MicrophysicsFinished,
                };
                observer(model_stage, view);
            },
        )?;
        copy_compact_active_to_padded(
            microphysics_shape,
            mass_shape,
            adapter_t2.values(),
            t2.values_mut(),
            active_west_east.clone(),
            active_south_north.clone(),
        );
        for (adapter, destination) in workspace
            .microphysics_moisture_fields
            .iter()
            .zip(&mut state.moisture_fields)
        {
            copy_compact_active_to_padded(
                microphysics_shape,
                mass_shape,
                adapter.values(),
                destination.values_mut(),
                active_west_east.clone(),
                active_south_north.clone(),
            );
        }
        Ok(())
    }
}

fn copy_state_to_microphysics_adapter(state: &ArwModelState, workspace: &mut ArwModelWorkspace) {
    let source_shape = state.mass_shape();
    let compact_shape = workspace.microphysics_shape();
    let [adapter_t2, adapter_al, adapter_alb, adapter_p, adapter_pb] =
        &mut workspace.microphysics_mass_fields;
    for (source, destination) in [
        state.mass_field_values(ArwMassField::CurrentPotentialTemperature),
        state.mass_field_values(ArwMassField::PerturbationInverseDensity),
        state.mass_field_values(ArwMassField::BaseInverseDensity),
        state.mass_field_values(ArwMassField::PerturbationPressure),
        state.mass_field_values(ArwMassField::BasePressure),
    ]
    .into_iter()
    .zip([adapter_t2, adapter_al, adapter_alb, adapter_p, adapter_pb])
    {
        copy_padded_mass_to_compact(
            source_shape,
            compact_shape,
            source,
            destination.values_mut(),
        );
    }
    let [adapter_ph2, adapter_phb] = &mut workspace.microphysics_geopotential_fields;
    for (source, destination) in [
        state.geopotential_field_values(ArwGeopotentialField::CurrentPerturbation),
        state.geopotential_field_values(ArwGeopotentialField::BaseState),
    ]
    .into_iter()
    .zip([adapter_ph2, adapter_phb])
    {
        copy_padded_w_to_compact(
            source_shape,
            compact_shape,
            source,
            destination.values_mut(),
        );
    }
    for (source, destination) in state
        .moisture_fields
        .iter()
        .zip(&mut workspace.microphysics_moisture_fields)
    {
        copy_padded_mass_to_compact(
            source_shape,
            compact_shape,
            source.values(),
            destination.values_mut(),
        );
    }
}

fn copy_padded_mass_to_compact(
    padded_shape: GridShape,
    compact_shape: GridShape,
    source: &[f32],
    destination: &mut [f32],
) {
    let west_east_points = padded_shape.west_east_points();
    for south_north in 0..padded_shape.south_north_points() {
        for compact_level in 0..compact_shape.bottom_top_points() {
            let padded_start = west_east_points
                * (compact_level + 1 + padded_shape.bottom_top_points() * south_north);
            let compact_start = west_east_points
                * (compact_level + compact_shape.bottom_top_points() * south_north);
            destination[compact_start..compact_start + west_east_points]
                .copy_from_slice(&source[padded_start..padded_start + west_east_points]);
        }
    }
}

fn copy_padded_w_to_compact(
    padded_mass_shape: GridShape,
    compact_mass_shape: GridShape,
    source: &[f32],
    destination: &mut [f32],
) {
    let west_east_points = padded_mass_shape.west_east_points();
    let padded_w_levels = padded_mass_shape.bottom_top_points();
    let compact_w_levels = compact_mass_shape.bottom_top_points() + 1;
    for south_north in 0..padded_mass_shape.south_north_points() {
        for compact_level in 0..compact_w_levels {
            let padded_start =
                west_east_points * (compact_level + 1 + padded_w_levels * south_north);
            let compact_start = west_east_points * (compact_level + compact_w_levels * south_north);
            destination[compact_start..compact_start + west_east_points]
                .copy_from_slice(&source[padded_start..padded_start + west_east_points]);
        }
    }
}

fn copy_compact_active_to_padded(
    compact_shape: GridShape,
    padded_shape: GridShape,
    source: &[f32],
    destination: &mut [f32],
    west_east_range: std::ops::Range<usize>,
    south_north_range: std::ops::Range<usize>,
) {
    let west_east_points = padded_shape.west_east_points();
    for south_north in south_north_range {
        for compact_level in 0..compact_shape.bottom_top_points() {
            let compact_start = west_east_points
                * (compact_level + compact_shape.bottom_top_points() * south_north)
                + west_east_range.start;
            let padded_start = west_east_points
                * (compact_level + 1 + padded_shape.bottom_top_points() * south_north)
                + west_east_range.start;
            let length = west_east_range.len();
            destination[padded_start..padded_start + length]
                .copy_from_slice(&source[compact_start..compact_start + length]);
        }
    }
}

fn copy_geopotential_to_dynamics(
    state: &ArwModelState,
    workspace: &mut ArwModelWorkspace,
) -> ArwModelResult<()> {
    for (state_field, workspace_index) in [
        (ArwGeopotentialField::PreviousPerturbation, 0),
        (ArwGeopotentialField::CurrentPerturbation, 1),
        (ArwGeopotentialField::BaseState, 2),
    ] {
        copy_w_field_to_common(
            state.mass_shape(),
            state.geopotential_field_values(state_field),
            workspace.volume_fields[workspace_index].values_mut(),
        );
    }
    Ok(())
}

fn copy_geopotential_to_registry(
    state: &mut ArwModelState,
    workspace: &ArwModelWorkspace,
) -> ArwModelResult<()> {
    let shape = state.mass_shape();
    copy_common_to_w_field(
        shape,
        workspace.volume_fields[1].values(),
        state.geopotential_field_values_mut(ArwGeopotentialField::CurrentPerturbation),
    );
    Ok(())
}

fn copy_w_field_to_common(shape: GridShape, source: &[f32], destination: &mut [f32]) {
    debug_assert_eq!(source.len(), shape.point_count());
    debug_assert_eq!(destination.len(), shape.point_count());
    destination.copy_from_slice(source);
}

fn copy_common_to_w_field(shape: GridShape, source: &[f32], destination: &mut [f32]) {
    debug_assert_eq!(source.len(), shape.point_count());
    debug_assert_eq!(destination.len(), shape.point_count());
    destination.copy_from_slice(source);
}

fn role_count(collection: &'static str) -> ArwModelError {
    ArwModelError::InternalRoleCountMismatch { collection }
}
