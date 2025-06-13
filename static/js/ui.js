export function showModal(modalId) {
    document.getElementById(modalId).style.display = 'flex';
}

export function hideModal(modalId, event) {
    if (event && event.target.id !== modalId) {
        return;
    }
    document.getElementById(modalId).style.display = 'none';
}

export function showScreen(screenId) {
    const screens = document.querySelectorAll('.screen');
    screens.forEach(screen => {
        screen.style.display = screen.id === screenId ? 'block' : 'none';
    });
}

export function copyToClipboard(text, successMessage) {
    navigator.clipboard.writeText(text).then(() => {
        showToast(successMessage || 'Copied to clipboard!');
    }).catch(err => {
        console.error('Failed to copy text: ', err);
        showToast('Failed to copy to clipboard.', 'error');
    });
}

export function showToast(message, type = 'info') {
    let toastContainer = document.getElementById('toast-container');
    if (!toastContainer) {
        toastContainer = document.createElement('div');
        toastContainer.id = 'toast-container';
        document.body.appendChild(toastContainer);
    }

    const toast = document.createElement('div');
    toast.className = `window toast ${type}`;

    const titleBar = document.createElement('div');
    titleBar.className = 'title-bar';

    const titleBarText = document.createElement('div');
    titleBarText.className = 'title-bar-text';
    titleBarText.textContent = type === 'error' ? 'Error' : 'Notification';

    const titleBarControls = document.createElement('div');
    titleBarControls.className = 'title-bar-controls';
    const closeButton = document.createElement('button');
    closeButton.setAttribute('aria-label', 'Close');
    closeButton.innerHTML = '<span></span>';
    closeButton.onclick = () => {
        toast.style.opacity = '0';
        toast.addEventListener('transitionend', () => toast.remove());
    };

    titleBarControls.appendChild(closeButton);
    titleBar.appendChild(titleBarText);
    titleBar.appendChild(titleBarControls);

    const windowBody = document.createElement('div');
    windowBody.className = 'window-body';
    windowBody.textContent = message;
    
    toast.appendChild(titleBar);
    toast.appendChild(windowBody);

    toastContainer.prepend(toast);

    setTimeout(() => {
        if (toast.parentElement) {
            toast.style.opacity = '0';
            toast.addEventListener('transitionend', () => toast.remove());
        }
    }, 5000);
} 