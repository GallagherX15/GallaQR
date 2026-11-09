// GallaQR Web Application
(function () {
  'use strict';

  // Referencias a elementos del DOM
  const qrTitleInput = document.getElementById('qr-title');
  const qrContentInput = document.getElementById('qr-content');
  const btnGenerate = document.getElementById('btn-generate');
  const qrColorPicker = document.getElementById('qr-color-picker');
  const qrColorHex = document.getElementById('qr-color-hex');
  const qrEccSelect = document.getElementById('qr-ecc');
  const qrMarginSelect = document.getElementById('qr-margin');
  const btnToggleBg = document.getElementById('btn-toggle-bg');
  const bgOptions = document.getElementById('bg-options');
  const bgColorPicker = document.getElementById('bg-color-picker');
  const bgColorHex = document.getElementById('bg-color-hex');
  const previewBox = document.getElementById('preview-box');
  const qrCanvas = document.getElementById('qr-canvas');
  const previewInfo = document.getElementById('preview-info');
  const exportWidthInput = document.getElementById('export-width');
  const exportHeightInput = document.getElementById('export-height');
  const lockAspectCheckbox = document.getElementById('lock-aspect');
  const toast = document.getElementById('toast');
  const sidebar = document.getElementById('sidebar');
  const sidebarOverlay = document.getElementById('sidebar-overlay');
  const btnToggleSidebar = document.getElementById('btn-toggle-sidebar');
  const btnCloseSidebar = document.getElementById('btn-close-sidebar');
  const historyList = document.getElementById('history-list');
  const historyCount = document.getElementById('history-count');
  const historySearch = document.getElementById('history-search');
  const btnNewQr = document.getElementById('btn-new-qr');
  const btnClearHistory = document.getElementById('btn-clear-history');

  // Botones de descarga y copiado
  const btnDownloadSvg = document.getElementById('btn-download-svg');
  const btnDownloadPng = document.getElementById('btn-download-png');
  const btnDownloadJpg = document.getElementById('btn-download-jpg');
  const btnCopySvg = document.getElementById('btn-copy-svg');
  const btnCopyPng = document.getElementById('btn-copy-png');

  // Estado de la aplicación
  let activeQr = null;
  let hasBackground = false;
  let qrColor = '#000000';
  let bgColor = '#FFFFFF';
  let selectedHistoryId = null;
  let toastTimeout = null;

  // Cargar historial de localStorage
  function loadHistory() {
    try {
      const stored = localStorage.getItem('gallaqr_history');
      return stored ? JSON.parse(stored) : [];
    } catch (e) {
      console.error('Error cargando historial:', e);
      return [];
    }
  }

  function saveHistory(items) {
    try {
      localStorage.setItem('gallaqr_history', JSON.stringify(items));
    } catch (e) {
      console.error('Error guardando historial:', e);
    }
  }

  let historyItems = loadHistory();

  // Mapeo de niveles de corrección de Nayuki
  function getEccEnum(code) {
    const QrCode = qrcodegen.QrCode;
    switch (code) {
      case 'L': return QrCode.Ecc.LOW;
      case 'Q': return QrCode.Ecc.QUARTILE;
      case 'H': return QrCode.Ecc.HIGH;
      default: return QrCode.Ecc.MEDIUM;
    }
  }

  // Notificación Toast
  function showToast(message, isSuccess = true) {
    if (toastTimeout) clearTimeout(toastTimeout);
    toast.className = `toast ${isSuccess ? 'success' : 'error'}`;
    toast.textContent = (isSuccess ? '✅ ' : '⚠️ ') + message;
    toast.classList.remove('hidden');

    toastTimeout = setTimeout(() => {
      toast.classList.add('hidden');
    }, 4500);
  }

  // Generar nombre de archivo amigable (slug)
  function getFileSlug() {
    const raw = qrTitleInput.value.trim() || 'gallaqr-code';
    return raw
      .toLowerCase()
      .replace(/[^a-z0-9_-]/g, '-')
      .replace(/-+/g, '-')
      .replace(/^-|-$/g, '') || 'gallaqr-code';
  }

  // Dibujar QR en un canvas HTML5
  function drawQrOnCanvas(qr, canvas, width, height, margin, fgColor, currentBgColor) {
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext('2d');
    ctx.clearRect(0, 0, width, height);

    if (currentBgColor) {
      ctx.fillStyle = currentBgColor;
      ctx.fillRect(0, 0, width, height);
    }

    const totalModules = qr.size + margin * 2;
    const scale = Math.min(width, height) / totalModules;
    const qrPx = scale * totalModules;
    const offsetX = (width - qrPx) / 2;
    const offsetY = (height - qrPx) / 2;

    ctx.fillStyle = fgColor;
    for (let y = 0; y < qr.size; y++) {
      for (let x = 0; x < qr.size; x++) {
        if (qr.getModule(x, y)) {
          const px = Math.round(offsetX + (x + margin) * scale);
          const py = Math.round(offsetY + (y + margin) * scale);
          const pw = Math.round(offsetX + (x + margin + 1) * scale) - px;
          const ph = Math.round(offsetY + (y + margin + 1) * scale) - py;
          ctx.fillRect(px, py, pw, ph);
        }
      }
    }
  }

  // Generar código SVG vectorial limpio
  function getSvgString(qr, width, height, margin, fgColor, currentBgColor) {
    const totalModules = qr.size + margin * 2;
    const scale = Math.min(width, height) / totalModules;
    const qrPx = scale * totalModules;
    const offsetX = (width - qrPx) / 2;
    const offsetY = (height - qrPx) / 2;

    let path = '';
    for (let y = 0; y < qr.size; y++) {
      for (let x = 0; x < qr.size; x++) {
        if (qr.getModule(x, y)) {
          const px = (offsetX + (x + margin) * scale).toFixed(2);
          const py = (offsetY + (y + margin) * scale).toFixed(2);
          const s = scale.toFixed(2);
          path += `M${px},${py}h${s}v${s}h-${s}z `;
        }
      }
    }

    const bgRect = currentBgColor
      ? `  <rect width="100%" height="100%" fill="${currentBgColor}" />\n`
      : '';

    return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" shape-rendering="crispEdges">
${bgRect}  <path d="${path}" fill="${fgColor}" />
</svg>`;
  }

  // Actualizar la vista previa
  function updatePreview() {
    if (!activeQr) {
      previewBox.classList.remove('has-qr');
      previewInfo.textContent = 'Matriz: 0x0 módulos';
      return;
    }

    previewBox.classList.add('has-qr');

    if (hasBackground) {
      previewBox.classList.remove('checkerboard');
      previewBox.style.backgroundColor = bgColor;
    } else {
      previewBox.classList.add('checkerboard');
      previewBox.style.backgroundColor = '';
    }

    const margin = parseInt(qrMarginSelect.value, 10);
    const renderBg = hasBackground ? bgColor : null;
    drawQrOnCanvas(activeQr, qrCanvas, 220, 220, margin, qrColor, renderBg);

    const bgLabel = hasBackground ? 'Con fondo' : 'Sin fondo (Transparente)';
    previewInfo.textContent = `Matriz: ${activeQr.size}x${activeQr.size} módulos • ${bgLabel}`;
  }

  // Generar el código QR actual
  function generateQr(saveToHistory = false) {
    const content = qrContentInput.value.trim();
    if (!content) {
      showToast('Ingresa un enlace o texto para generar el QR', false);
      return;
    }

    try {
      const ecc = getEccEnum(qrEccSelect.value);
      activeQr = qrcodegen.QrCode.encodeText(content, ecc);
      updatePreview();

      if (saveToHistory) {
        const title = qrTitleInput.value.trim() || 'Sin Título';
        const newItem = {
          id: Date.now().toString(),
          title: title,
          content: content,
          ecc: qrEccSelect.value,
          color: qrColor,
          bgColor: bgColor,
          hasBg: hasBackground,
          createdAt: new Date().toLocaleDateString('es-ES', {
            day: '2-digit',
            month: '2-digit',
            year: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
          })
        };

        historyItems.unshift(newItem);
        saveHistory(historyItems);
        renderHistory();
        selectedHistoryId = newItem.id;
        showToast('✨ ¡Código QR generado y guardado en el historial!');
      }
    } catch (e) {
      console.error(e);
      showToast('Error generando el código QR: ' + e.message, false);
    }
  }

  // Renderizar historial
  function renderHistory() {
    historyCount.textContent = historyItems.length;
    const query = historySearch.value.trim().toLowerCase();
    const filtered = historyItems.filter(it =>
      it.title.toLowerCase().includes(query) || it.content.toLowerCase().includes(query)
    );

    historyList.innerHTML = '';

    if (filtered.length === 0) {
      const emptyMsg = document.createElement('div');
      emptyMsg.style.cssText = 'padding: 30px 10px; text-align: center; font-size: 12.5px; color: var(--text-dim);';
      emptyMsg.textContent = historyItems.length === 0
        ? 'Aún no tienes QRs guardados. Genera el primero en el formulario.'
        : 'Sin resultados de búsqueda.';
      historyList.appendChild(emptyMsg);
      return;
    }

    filtered.forEach(item => {
      const card = document.createElement('div');
      card.className = `history-card ${selectedHistoryId === item.id ? 'selected' : ''}`;

      card.innerHTML = `
        <div class="history-card-top">
          <span class="history-card-title">${escapeHtml(item.title)}</span>
          <button class="btn-delete-item" title="Eliminar del historial">🗑</button>
        </div>
        <div class="history-card-content">${escapeHtml(item.content)}</div>
        <div class="history-card-meta">
          <span>${item.createdAt}</span>
          <span class="history-card-ecc">ECC: ${item.ecc}</span>
        </div>
      `;

      // Clic para cargar
      card.addEventListener('click', (e) => {
        if (e.target.closest('.btn-delete-item')) return;
        loadHistoryItem(item);
      });

      // Eliminar item
      card.querySelector('.btn-delete-item').addEventListener('click', (e) => {
        e.stopPropagation();
        historyItems = historyItems.filter(it => it.id !== item.id);
        saveHistory(historyItems);
        if (selectedHistoryId === item.id) selectedHistoryId = null;
        renderHistory();
        showToast('Elemento eliminado del historial');
      });

      historyList.appendChild(card);
    });
  }

  function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }

  function loadHistoryItem(item) {
    selectedHistoryId = item.id;
    qrTitleInput.value = item.title;
    qrContentInput.value = item.content;
    qrEccSelect.value = item.ecc;
    qrColor = item.color || '#000000';
    qrColorPicker.value = qrColor;
    qrColorHex.value = qrColor.toUpperCase();

    if (item.hasBg) {
      hasBackground = true;
      bgColor = item.bgColor || '#FFFFFF';
      bgColorPicker.value = bgColor;
      bgColorHex.value = bgColor.toUpperCase();
      btnToggleBg.classList.add('active');
      btnToggleBg.textContent = '⬜ Con Fondo';
      bgOptions.classList.remove('hidden');
    } else {
      hasBackground = false;
      btnToggleBg.classList.remove('active');
      btnToggleBg.textContent = '🔳 Sin Fondo (Transparente)';
      bgOptions.classList.add('hidden');
    }

    renderHistory();
    generateQr(false);

    // En móviles, cerrar automáticamente el sidebar al seleccionar un elemento
    if (window.innerWidth <= 860) {
      closeSidebar();
    }
  }

  function clearForm() {
    selectedHistoryId = null;
    qrTitleInput.value = '';
    qrContentInput.value = '';
    activeQr = null;
    updatePreview();
    renderHistory();
    showToast('Formulario limpio. Listo para un nuevo QR.');
  }

  // Sincronización de colores HEX y Color Picker
  function syncColorInput(picker, hexInput, onChange) {
    picker.addEventListener('input', () => {
      hexInput.value = picker.value.toUpperCase();
      onChange(picker.value);
    });

    hexInput.addEventListener('input', () => {
      let val = hexInput.value.trim();
      if (!val.startsWith('#')) val = '#' + val;
      if (/^#([0-9A-F]{3}){1,2}$/i.test(val)) {
        picker.value = val;
        onChange(val);
      }
    });
  }

  syncColorInput(qrColorPicker, qrColorHex, (col) => {
    qrColor = col;
    if (activeQr) updatePreview();
  });

  syncColorInput(bgColorPicker, bgColorHex, (col) => {
    bgColor = col;
    if (activeQr && hasBackground) updatePreview();
  });

  // Swatches rápidos
  document.getElementById('qr-swatches').addEventListener('click', (e) => {
    const swatch = e.target.closest('.swatch');
    if (!swatch) return;
    const col = swatch.getAttribute('data-color');
    qrColor = col;
    qrColorPicker.value = col;
    qrColorHex.value = col.toUpperCase();
    if (activeQr) updatePreview();
  });

  document.getElementById('bg-swatches').addEventListener('click', (e) => {
    const swatch = e.target.closest('.swatch');
    if (!swatch) return;
    const col = swatch.getAttribute('data-color');
    bgColor = col;
    bgColorPicker.value = col;
    bgColorHex.value = col.toUpperCase();
    if (activeQr && hasBackground) updatePreview();
  });

  // Toggle de Fondo
  btnToggleBg.addEventListener('click', () => {
    hasBackground = !hasBackground;
    if (hasBackground) {
      btnToggleBg.classList.add('active');
      btnToggleBg.textContent = '⬜ Con Fondo';
      bgOptions.classList.remove('hidden');
    } else {
      btnToggleBg.classList.remove('active');
      btnToggleBg.textContent = '🔳 Sin Fondo (Transparente)';
      bgOptions.classList.add('hidden');
    }
    if (activeQr) updatePreview();
  });

  // Presets de resolución 1:1
  document.getElementById('size-chips').addEventListener('click', (e) => {
    const chip = e.target.closest('.chip');
    if (!chip) return;
    document.querySelectorAll('.chip').forEach(c => c.classList.remove('active'));
    chip.classList.add('active');
    const size = parseInt(chip.getAttribute('data-size'), 10);
    exportWidthInput.value = size;
    exportHeightInput.value = size;
  });

  // Bloqueo de proporción 1:1
  exportWidthInput.addEventListener('input', () => {
    if (lockAspectCheckbox.checked) {
      exportHeightInput.value = exportWidthInput.value;
    }
  });

  exportHeightInput.addEventListener('input', () => {
    if (lockAspectCheckbox.checked) {
      exportWidthInput.value = exportHeightInput.value;
    }
  });

  // Descargas
  function triggerDownload(blob, filename) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  btnDownloadSvg.addEventListener('click', () => {
    if (!activeQr) return showToast('Genera un QR antes de descargar', false);
    const width = parseInt(exportWidthInput.value, 10) || 1024;
    const height = parseInt(exportHeightInput.value, 10) || 1024;
    const margin = parseInt(qrMarginSelect.value, 10);
    const renderBg = hasBackground ? bgColor : null;

    const svg = getSvgString(activeQr, width, height, margin, qrColor, renderBg);
    const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' });
    triggerDownload(blob, `${getFileSlug()}.svg`);
    showToast('✅ Archivo SVG descargado');
  });

  btnDownloadPng.addEventListener('click', () => {
    if (!activeQr) return showToast('Genera un QR antes de descargar', false);
    const width = parseInt(exportWidthInput.value, 10) || 1024;
    const height = parseInt(exportHeightInput.value, 10) || 1024;
    const margin = parseInt(qrMarginSelect.value, 10);
    const renderBg = hasBackground ? bgColor : null;

    const offscreen = document.createElement('canvas');
    drawQrOnCanvas(activeQr, offscreen, width, height, margin, qrColor, renderBg);

    offscreen.toBlob((blob) => {
      if (blob) {
        triggerDownload(blob, `${getFileSlug()}.png`);
        showToast(`✅ PNG (${width}x${height} px) descargado`);
      }
    }, 'image/png');
  });

  btnDownloadJpg.addEventListener('click', () => {
    if (!activeQr) return showToast('Genera un QR antes de descargar', false);
    const width = parseInt(exportWidthInput.value, 10) || 1024;
    const height = parseInt(exportHeightInput.value, 10) || 1024;
    const margin = parseInt(qrMarginSelect.value, 10);
    // JPG siempre lleva fondo sólido
    const renderBg = hasBackground ? bgColor : '#FFFFFF';

    const offscreen = document.createElement('canvas');
    drawQrOnCanvas(activeQr, offscreen, width, height, margin, qrColor, renderBg);

    offscreen.toBlob((blob) => {
      if (blob) {
        triggerDownload(blob, `${getFileSlug()}.jpg`);
        showToast(`✅ JPG (${width}x${height} px) descargado`);
      }
    }, 'image/jpeg', 0.95);
  });

  // Copiar al portapapeles
  btnCopySvg.addEventListener('click', async () => {
    if (!activeQr) return showToast('Genera un QR antes de copiar', false);
    const width = parseInt(exportWidthInput.value, 10) || 1024;
    const height = parseInt(exportHeightInput.value, 10) || 1024;
    const margin = parseInt(qrMarginSelect.value, 10);
    const renderBg = hasBackground ? bgColor : null;

    const svg = getSvgString(activeQr, width, height, margin, qrColor, renderBg);
    try {
      await navigator.clipboard.writeText(svg);
      showToast('📋 ¡SVG vectorial copiado al portapapeles!');
    } catch (e) {
      showToast('No se pudo copiar al portapapeles', false);
    }
  });

  btnCopyPng.addEventListener('click', async () => {
    if (!activeQr) return showToast('Genera un QR antes de copiar', false);
    const width = parseInt(exportWidthInput.value, 10) || 1024;
    const height = parseInt(exportHeightInput.value, 10) || 1024;
    const margin = parseInt(qrMarginSelect.value, 10);
    const renderBg = hasBackground ? bgColor : null;

    const offscreen = document.createElement('canvas');
    drawQrOnCanvas(activeQr, offscreen, width, height, margin, qrColor, renderBg);

    offscreen.toBlob(async (blob) => {
      if (blob) {
        try {
          await navigator.clipboard.write([
            new ClipboardItem({ 'image/png': blob })
          ]);
          showToast('📋 ¡Imagen PNG copiada al portapapeles!');
        } catch (e) {
          showToast('No se pudo copiar la imagen al portapapeles en este navegador', false);
        }
      }
    }, 'image/png');
  });

  // Botón Generar
  btnGenerate.addEventListener('click', () => generateQr(true));

  // Cambios de ECC y Margen
  qrEccSelect.addEventListener('change', () => { if (activeQr) generateQr(false); });
  qrMarginSelect.addEventListener('change', () => { if (activeQr) updatePreview(); });

  // Control responsive del Sidebar (móvil y escritorio)
  function openSidebar() {
    if (window.innerWidth <= 860) {
      sidebar.classList.add('mobile-open');
      sidebarOverlay.classList.add('active');
    } else {
      sidebar.classList.remove('collapsed');
    }
  }

  function closeSidebar() {
    if (window.innerWidth <= 860) {
      sidebar.classList.remove('mobile-open');
      sidebarOverlay.classList.remove('active');
    } else {
      sidebar.classList.add('collapsed');
    }
  }

  function toggleSidebar() {
    if (window.innerWidth <= 860) {
      if (sidebar.classList.contains('mobile-open')) {
        closeSidebar();
      } else {
        openSidebar();
      }
    } else {
      sidebar.classList.toggle('collapsed');
    }
  }

  btnToggleSidebar.addEventListener('click', toggleSidebar);
  btnCloseSidebar.addEventListener('click', closeSidebar);
  sidebarOverlay.addEventListener('click', closeSidebar);

  // Botón nuevo QR
  btnNewQr.addEventListener('click', clearForm);

  // Búsqueda en historial
  historySearch.addEventListener('input', renderHistory);

  // Limpiar historial completo
  btnClearHistory.addEventListener('click', () => {
    if (confirm('¿Deseas vaciar todo el historial de QRs?')) {
      historyItems = [];
      saveHistory([]);
      renderHistory();
      showToast('Historial vaciado');
    }
  });

  // Inicialización
  renderHistory();
  if (historyItems.length > 0) {
    loadHistoryItem(historyItems[0]);
  } else {
    // QR por defecto de bienvenida
    qrTitleInput.value = 'Mi Portafolio Web';
    qrContentInput.value = 'https://github.com/GallagherX15/GallaQR';
    generateQr(false);
  }
})();
