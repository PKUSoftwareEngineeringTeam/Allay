/**
 * A client for live reloading web pages based on file changes.
 * It periodically polls the server to check for any modifications in the files,
 * and if changes are detected, it reloads the page after showing a notification
 * about which files have been changed.
 */
class LiveReloadClient {
    constructor() {
        this.pollingInterval = 50; // 50 ms
        this.lastTimestamps = new Map();
        this.init().then();
    }

    async init() {
        console.log('Live reload client initialized');
        await this.fetchInitialTimestamps();
        this.startPolling();
    }

    /**
     * Fetches the initial timestamps for files and stores them.
     * This method sends a request to the server to get the last modified timestamps of files,
     * then updates the `lastTimestamps` with the received data.
     */
    async fetchInitialTimestamps() {
        try {
            const response = await fetch('/api/last-modified');
            const timestamps = await response.json();

            for (const [file, timestamp] of Object.entries(timestamps)) {
                this.lastTimestamps.set(file, timestamp);
            }

            console.log('Initial file timestamps loaded');
        } catch (error) {
            console.error('Failed to fetch initial timestamps:', error);
        }
    }

    startPolling() {
        setInterval(async () => {
            await this.checkForChanges();
        }, this.pollingInterval);
    }

    /**
     * Asynchronously checks for changes in the files by comparing the last modified timestamps.
     * If any file has been modified or deleted, it logs the changes, shows a reload notification,
     * and reloads the page after a short delay.
     */
    async checkForChanges() {
        try {
            const response = await fetch('/api/last-modified');
            const currentTimestamps = await response.json();

            let shouldReload = false;
            let changedFiles = [];

            for (const [file, timestamp] of Object.entries(currentTimestamps)) {
                const lastTimestamp = this.lastTimestamps.get(file);

                if (!lastTimestamp || lastTimestamp < timestamp) {
                    shouldReload = true;
                    changedFiles.push(file);
                    this.lastTimestamps.set(file, timestamp);
                }
            }

            for (const file of this.lastTimestamps.keys()) {
                if (!currentTimestamps[file]) {
                    shouldReload = true;
                    changedFiles.push(`${file} (deleted)`);
                    this.lastTimestamps.delete(file);
                }
            }

            if (shouldReload) {
                console.log('Files changed, reloading page:', changedFiles);
                window.location.reload();
            }
        } catch (error) {
            console.error('Error checking for changes:', error);
        }
    }
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        new LiveReloadClient();
    });
} else {
    new LiveReloadClient();
}