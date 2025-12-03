/**
 * A client for live reloading web pages based on file changes.
 * It periodically polls the server to check for any modifications in the files,
 * and if changes are detected, it reloads the page after showing a notification
 * about which files have been changed.
 */
class LiveReloadClient {
    constructor() {
        this.pollingInterval = 500; // 500 ms
        this.lastTimestamp = 0;
        this.init().then();
    }

    async init() {
        this.lastTimestamp = await this.fetchTimestamp();
        console.log("Live Reload Client initialized with timestamp:", this.lastTimestamp);
        this.startPolling();
    }

    /**
     * Fetches the last modified timestamp for the current URI from the server.
     * @returns {Promise<number>} The last modified timestamp.
     */
    async fetchTimestamp() {
        try {
            let url = location.pathname;
            if (url.startsWith('/')) {
                url = url.slice(1);
            }
            const response = await fetch('/api/last-modified?url=' + encodeURIComponent(url));
            return await response.json();
        } catch (error) {
            console.error('Failed to fetch timestamps:', error);
            return 0;
        }
    }

    /**
     * Starts polling the server at regular intervals to check for file changes.
     */
    startPolling() {
        setInterval(async () => {
            await this.checkForChange();
        }, this.pollingInterval);
    }

    /**
     * Asynchronously checks for change in the uri by comparing the last modified timestamp.
     */
    async checkForChange() {
        try {
            const currentTimestamp = await this.fetchTimestamp();

            if (currentTimestamp > this.lastTimestamp) {
                this.lastTimestamp = currentTimestamp;
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