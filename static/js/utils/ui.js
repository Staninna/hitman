function showToast(message, type = 'info', duration = 6000) {
    const toastContainer = document.createElement('div');
    toastContainer.className = 'toast-container';

    const toastWindow = document.createElement('div');
    toastWindow.className = 'window';

    const titleBar = document.createElement('div');
    titleBar.className = 'title-bar';

    const titleBarText = document.createElement('div');
    titleBarText.className = 'title-bar-text';
    titleBarText.textContent = type === 'error' ? 'Error' : 'Notification';

    const titleBarControls = document.createElement('div');
    titleBarControls.className = 'title-bar-controls';
    
    const closeButton = document.createElement('button');
    closeButton.setAttribute('aria-label', 'Close');
    
    titleBarControls.appendChild(closeButton);
    titleBar.appendChild(titleBarText);
    titleBar.appendChild(titleBarControls);

    const windowBody = document.createElement('div');
    windowBody.className = 'window-body';
    windowBody.textContent = message;

    toastWindow.appendChild(titleBar);
    toastWindow.appendChild(windowBody);
    
    toastContainer.appendChild(toastWindow);
    document.body.appendChild(toastContainer);

    const dismiss = () => {
        toastContainer.classList.remove('show');
        setTimeout(() => {
            if (document.body.contains(toastContainer)) {
                document.body.removeChild(toastContainer);
            }
        }, 500); // Match CSS transition
    };

    closeButton.addEventListener('click', dismiss);
    const timeoutId = setTimeout(dismiss, duration);

    setTimeout(() => {
        toastContainer.classList.add('show');
    }, 100);
}

function copyToClipboard(text, successMessage) {
    navigator.clipboard.writeText(text).then(function () {
        showToast(successMessage);
    }, function (err) {
        console.error('Could not copy text: ', err);
        showToast('Failed to copy text', 'error');
    });
}

function showScreen(screenId) {
    const screens = document.querySelectorAll('.screen');
    screens.forEach(screen => {
        screen.style.display = 'none';
    });
    const activeScreen = document.getElementById(screenId);
    if (activeScreen) {
        activeScreen.style.display = 'block';
    }
}

function showModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.classList.remove('hidden');
    }
}

function hideModal(modalId) {
    const modal = document.getElementById(modalId);
    if (modal) {
        modal.classList.add('hidden');
    }
}

export {
    showToast,
    copyToClipboard,
    showScreen,
    showModal,
    hideModal,
}; 