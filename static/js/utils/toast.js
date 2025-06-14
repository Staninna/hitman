function showToast(message, type = 'info', duration = 3000) {
    // This is the container for positioning and animation
    const toastContainer = document.createElement('div');
    toastContainer.className = 'toast-container';

    // This is the actual XP.css window
    const toastWindow = document.createElement('div');
    toastWindow.className = 'window';

    // 1. Create Title Bar
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

    // 2. Create Window Body
    const windowBody = document.createElement('div');
    windowBody.className = 'window-body';
    windowBody.textContent = message;

    // 3. Assemble the toast window
    toastWindow.appendChild(titleBar);
    toastWindow.appendChild(windowBody);
    
    // 4. Add the window to the container, and the container to the body
    toastContainer.appendChild(toastWindow);
    document.body.appendChild(toastContainer);

    // 5. Handle dismissal
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

    // 6. Show the toast
    setTimeout(() => {
        toastContainer.classList.add('show');
    }, 100);
}

export { showToast }; 