program vertical_acoustic_coefficients_driver
  use iso_fortran_env, only: int32
  use extracted_vertical_acoustic_coefficients, only: calc_coef_w
  implicit none
  call run_case('open_full',.false.,0,4,1,5,.false.)
  call run_case('rigid_full',.true.,0,4,1,5,.false.)
  call run_case('open_partial',.false.,1,2,2,3,.true.)
  call run_case('rigid_partial',.true.,1,2,2,3,.true.)
contains
  subroutine run_case(name,top_lid,its,ite,jts,jte,exceptional)
    character(len=*),intent(in)::name
    logical,intent(in)::top_lid,exceptional
    integer,intent(in)::its,ite,jts,jte
    integer,parameter::ims=-1,ime=4,jms=0,jme=5,kms=-1,kme=5
    integer,parameter::ids=0,ide=4,jds=1,jde=5,kds=1,kde=4
    integer,parameter::kts=2,kte=2
    real::a(ims:ime,kms:kme,jms:jme)
    real::alpha(ims:ime,kms:kme,jms:jme)
    real::gamma(ims:ime,kms:kme,jms:jme)
    real::mut(ims:ime,jms:jme)
    real::cqw(ims:ime,kms:kme,jms:jme)
    real::c2a(ims:ime,kms:kme,jms:jme)
    real::c1h(kms:kme),c2h(kms:kme),c1f(kms:kme),c2f(kms:kme)
    real::c3h(kms:kme),c4h(kms:kme),c3f(kms:kme),c4f(kms:kme)
    real::rdn(kms:kme),rdnw(kms:kme)
    real,parameter::dts=2.5,g=9.81,epssm=.1
    integer::i,j,k
    do k=kms,kme
      c1h(k)=.2+real(k)*.03
      c2h(k)=.4-real(k)*.02
      c1f(k)=.25+real(k)*.015
      c2f(k)=.35-real(k)*.01
      rdn(k)=1.1+real(k)*.04
      rdnw(k)=1.3+real(k)*.05
      c3h(k)=7.;c4h(k)=8.;c3f(k)=9.;c4f(k)=10.
    enddo
    do j=jms,jme
      do i=ims,ime
        mut(i,j)=40.+real(i)*1.3+real(j)*.7
      enddo
      do k=kms,kme
        do i=ims,ime
          cqw(i,k,j)=.9+real(i)*.01-real(k)*.02+real(j)*.015
          c2a(i,k,j)=140000.+real(i)*13.+real(k)*17.+real(j)*11.
        enddo
      enddo
    enddo
    a=-901.;alpha=-902.;gamma=-903.
    if(exceptional)then
      c1h(2)=0.;c2h(2)=0.
      c1f(3)=0.;c2f(3)=0.
      cqw(1,2,2)=-0.
      c2a(2,2,2)=0.
      c2a(1,3,2)=huge(c2a)*2.
    endif
    call calc_coef_w(a,alpha,gamma,mut,c1h,c2h,c1f,c2f,c3h,c4h,c3f,c4f, &
      cqw,rdn,rdnw,c2a,dts,g,epssm,top_lid, &
      ids,ide,jds,jde,kds,kde,ims,ime,jms,jme,kms,kme, &
      its,ite,jts,jte,kts,kte)
    call write_volume(name,'a',a)
    call write_volume(name,'alpha',alpha)
    call write_volume(name,'gamma',gamma)
  end subroutine

  subroutine write_volume(case_name,field_name,field)
    character(len=*),intent(in)::case_name,field_name
    real,intent(in)::field(-1:4,-1:5,0:5)
    integer::i,j,k
    do j=0,5;do k=-1,5;do i=-1,4
      if(isnan(field(i,k,j)))then
        write(*,'(A,1X,A,3(1X,I0),1X,A)')case_name,field_name,i+1,k+1,j,'NAN'
      else
        write(*,'(A,1X,A,3(1X,I0),1X,Z8.8)')case_name,field_name,i+1,k+1,j,transfer(field(i,k,j),0_int32)
      endif
    enddo;enddo;enddo
  end subroutine
end program
