/**
 * Vani Protocol JavaScript SDK
 * 
 * A simple SDK for integrating vernacular crypto operations
 * into your applications using the Vani API.
 */

class VaniSDK {
  constructor(config = {}) {
    this.apiKey = config.apiKey || process.env.VANI_API_KEY;
    this.apiURL = config.apiURL || 'http://localhost:8080';
    this.timeout = config.timeout || 10000; // 10 seconds default
  }

  /**
   * Parse a vernacular command into structured intent
   * 
   * @param {string} text - The vernacular command text
   * @param {string} language - Language code (hindi, telugu, tamil, english)
   * @returns {Promise<Object>} Parse response with action, source, target, amount
   */
  async parseCommand(text, language = 'hindi') {
    const response = await this._request('/api/parse', {
      text,
      language
    });
    return response;
  }

  /**
   * Execute a parsed intent as a blockchain transaction
   * 
   * @param {Object} intent - The parsed intent from parseCommand
   * @param {string} walletAddress - Optional wallet address for execution
   * @returns {Promise<Object>} Execution response with transaction ID
   */
  async executeTransaction(intent, walletAddress = null) {
    const response = await this._request('/api/execute', {
      intent,
      wallet_address: walletAddress
    });
    return response;
  }

  /**
   * Combined parse and execute in one call
   * 
   * @param {string} text - The vernacular command text
   * @param {string} language - Language code
   * @param {string} walletAddress - Optional wallet address
   * @returns {Promise<Object>} Execution response
   */
  async parseAndExecute(text, language = 'hindi', walletAddress = null) {
    const intent = await this.parseCommand(text, language);
    return await this.executeTransaction(intent, walletAddress);
  }

  /**
   * Check API health status
   * 
   * @returns {Promise<string>} Health status message
   */
  async health() {
    const response = await fetch(`${this.apiURL}/health`);
    return await response.text();
  }

  /**
   * Internal HTTP request helper
   */
  async _request(endpoint, data) {
    const url = `${this.apiURL}${endpoint}`;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(this.apiKey && { 'Authorization': `Bearer ${this.apiKey}` })
        },
        body: JSON.stringify(data),
        signal: controller.signal
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        throw new Error(`Vani API error: ${response.status} ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      throw error;
    }
  }
}

// Export both default and named exports
export default VaniSDK;
export { VaniSDK };