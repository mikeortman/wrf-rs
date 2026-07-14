(() => {
  const navToggle = document.querySelector("[data-nav-toggle]");
  const navLinks = document.querySelector("[data-nav-links]");
  navToggle?.addEventListener("click", () => {
    const isOpen = navLinks.classList.toggle("is-open");
    navToggle.setAttribute("aria-expanded", String(isOpen));
  });

  const search = document.querySelector("[data-benchmark-search]");
  const status = document.querySelector("[data-status-filter]");
  const benchmarkRows = [...document.querySelectorAll("[data-benchmark-row]")];
  const emptyState = document.querySelector("[data-empty-state]");
  const filterRows = () => {
    const query = search?.value.trim().toLowerCase() ?? "";
    const selectedStatus = status?.value ?? "all";
    let visible = 0;
    for (const row of benchmarkRows) {
      const matchesText = row.dataset.label.includes(query);
      const matchesStatus = selectedStatus === "all" || row.dataset.status === selectedStatus;
      row.hidden = !(matchesText && matchesStatus);
      if (!row.hidden) visible += 1;
    }
    if (emptyState) emptyState.style.display = visible ? "none" : "block";
  };
  search?.addEventListener("input", filterRows);
  status?.addEventListener("change", filterRows);

  const docsSearch = document.querySelector("[data-docs-search]");
  const docsLinks = [...document.querySelectorAll("[data-doc-link]")];
  docsSearch?.addEventListener("input", () => {
    const query = docsSearch.value.trim().toLowerCase();
    for (const link of docsLinks) {
      link.hidden = !link.textContent.toLowerCase().includes(query);
    }
  });

  const colors = { p50: "#56d6e7", p90: "#f8c66a", p99: "#ff7185" };
  const drawChart = (canvas) => {
    const points = JSON.parse(canvas.dataset.history || "[]");
    const ratio = window.devicePixelRatio || 1;
    const width = Math.max(canvas.clientWidth, 320);
    const height = Math.max(canvas.clientHeight, 220);
    canvas.width = width * ratio;
    canvas.height = height * ratio;
    const context = canvas.getContext("2d");
    context.scale(ratio, ratio);
    context.clearRect(0, 0, width, height);
    const padding = { top: 18, right: 18, bottom: 34, left: 48 };
    const chartWidth = width - padding.left - padding.right;
    const chartHeight = height - padding.top - padding.bottom;
    const values = points.flatMap((point) => [point.p50, point.p90, point.p99]).filter(Number.isFinite);
    if (!values.length) {
      context.fillStyle = "#95a4b8";
      context.font = "13px system-ui";
      context.fillText("History begins with the next benchmark run.", padding.left, padding.top + 24);
      return;
    }
    const minimum = Math.max(0, Math.min(...values, 1) * 0.88);
    const maximum = Math.max(...values, 1) * 1.12;
    const x = (index) => padding.left + (points.length === 1 ? chartWidth / 2 : index * chartWidth / (points.length - 1));
    const y = (value) => padding.top + (maximum - value) * chartHeight / (maximum - minimum || 1);
    context.font = "11px system-ui";
    context.textAlign = "right";
    context.fillStyle = "#68788e";
    context.strokeStyle = "#172b3a";
    context.lineWidth = 1;
    for (let step = 0; step <= 4; step += 1) {
      const value = minimum + (maximum - minimum) * step / 4;
      const lineY = y(value);
      context.beginPath();
      context.moveTo(padding.left, lineY);
      context.lineTo(width - padding.right, lineY);
      context.stroke();
      context.fillText(`${value.toFixed(2)}×`, padding.left - 8, lineY + 4);
    }
    context.strokeStyle = "#54dc93";
    context.setLineDash([5, 5]);
    context.beginPath();
    context.moveTo(padding.left, y(1));
    context.lineTo(width - padding.right, y(1));
    context.stroke();
    context.setLineDash([]);
    for (const percentile of ["p50", "p90", "p99"]) {
      context.strokeStyle = colors[percentile];
      context.fillStyle = colors[percentile];
      context.lineWidth = 2;
      context.beginPath();
      let started = false;
      points.forEach((point, index) => {
        if (!Number.isFinite(point[percentile])) return;
        if (started) context.lineTo(x(index), y(point[percentile]));
        else context.moveTo(x(index), y(point[percentile]));
        started = true;
      });
      context.stroke();
      points.forEach((point, index) => {
        if (!Number.isFinite(point[percentile])) return;
        context.beginPath();
        context.arc(x(index), y(point[percentile]), 3, 0, Math.PI * 2);
        context.fill();
      });
    }
    context.textAlign = "center";
    context.fillStyle = "#68788e";
    const labels = points.length <= 5 ? points.map((_, index) => index) : [0, Math.floor((points.length - 1) / 2), points.length - 1];
    for (const index of labels) context.fillText(points[index].date, x(index), height - 8);
  };

  const charts = [...document.querySelectorAll("[data-performance-chart]")];
  const drawCharts = () => charts.forEach(drawChart);
  drawCharts();
  let resizeTimer;
  window.addEventListener("resize", () => {
    window.clearTimeout(resizeTimer);
    resizeTimer = window.setTimeout(drawCharts, 100);
  });
})();
