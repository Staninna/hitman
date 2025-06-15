import { showToast } from "../utils/ui.js";

let html5QrScanner = null;

function onElementReady(selectors, callback) {
    const selectorArray = Array.isArray(selectors) ? selectors : [selectors];

    const intervalId = setInterval(() => {
        const elements = selectorArray.map((s) => document.querySelector(s));

        if (elements.every((el) => el !== null)) {
            clearInterval(intervalId);
            clearTimeout(timeoutId);
            callback(elements);
        }
    }, 50);

    const timeoutId = setTimeout(() => {
        clearInterval(intervalId);
        console.error(
            `One or more elements not found within 10s: ${selectorArray.join(
                ", "
            )}`
        );
    }, 10000);
}

function alignVideoWithOverlay(videoElement, shadedRegion) {
    const regionStyles = window.getComputedStyle(shadedRegion);
    const stylesToApply = {
        position: "absolute",
        top: regionStyles.top,
        left: regionStyles.left,
        width: regionStyles.width,
        height: regionStyles.height,
        objectFit: "cover",
        zIndex: "0",
    };
    Object.assign(videoElement.style, stylesToApply);
}

export function stopScanner() {
    const overlay = document.getElementById("qrScannerOverlay");
    if (overlay) {
        overlay.style.display = "none";
    }

    if (html5QrScanner && html5QrScanner.isScanning) {
        html5QrScanner
            .stop()
            .catch((err) => console.warn("Failed to stop scanner:", err));
    }
}

export function startScanner(onScanSuccess) {
    const overlay = document.getElementById("qrScannerOverlay");
    if (!overlay) {
        console.error("Scanner overlay element not found!");
        return;
    }
    overlay.style.display = "flex";

    if (!html5QrScanner) {
        html5QrScanner = new Html5Qrcode("qrReader");
    }

    const successCallback = (decodedText) => {
        stopScanner();
        onScanSuccess(decodedText);
    };

    html5QrScanner
        .start(
            { facingMode: "environment" },
            { fps: 10, qrbox: { width: 250, height: 250 } },
            successCallback,
            () => {} // Ignore scan failure callback
        )
        .catch((err) => {
            console.error("Failed to start QR scanner:", err);
            showToast("Unable to start camera for scanning.", "error");
            stopScanner();
        });

    onElementReady(
        ["#qrReader > video", "#qr-shaded-region"],
        ([videoElement, shadedRegion]) => {
            alignVideoWithOverlay(videoElement, shadedRegion);
        }
    );
}