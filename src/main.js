// Complete File Override: d:/Projects/Trading/src/main.js
document.addEventListener('DOMContentLoaded', () => {
    // --- 1. Defensive Direct-ID Eye Reveal Functionality Matrix ---
    const bindVectorEyeToggle = (btnId, inputId, pathId) => {
        const btn = document.getElementById(btnId);
        const input = document.getElementById(inputId);
        const path = document.getElementById(pathId);
        const SVG_OPEN = "M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178Z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z";
        const SVG_SLASH = "M3.98 8.223A10.477 10.477 0 0 0 1.934 11.66a1.014 1.014 0 0 0 0 .68c1.373 4.12 5.222 7.16 9.816 7.16 1.411 0 2.763-.292 3.99-.817m-1.464-1.464A4.975 4.975 0 0 1 11.75 17.25a5 5 0 0 1-5-5c0-1.04.319-2.008.863-2.813M14.852 14.852l4.912 4.912m-4.912-4.912A4.975 4.975 0 0 0 16.75 12.25a5 5 0 0 0-5-5c-1.04 0-2.008.319-2.813.863m5.913 6.739 4.913 4.913M9.13 9.13l4.912 4.912M9.13 9.13A4.975 4.975 0 0 1 6.75 12.25a5 5 0 0 1 5-5c1.04 0 2.008.319 2.813.863M21.566 11.66a10.475 10.475 0 0 0-2.046-3.437M19.52 19.52l-15-15";

        if (btn && input && path) {
            btn.addEventListener('click', (e) => {
                e.preventDefault();
                if (input.type === 'password') {
                    input.type = 'text';
                    path.setAttribute('d', SVG_SLASH);
                } else {
                    input.type = 'password';
                    path.setAttribute('d', SVG_OPEN);
                }
            });
        }
    };
    bindVectorEyeToggle('toggle-api-key', 'api-key', 'path-api-key');
    bindVectorEyeToggle('toggle-secret-key', 'secret-key', 'path-secret-key');
    bindVectorEyeToggle('toggle-password', 'acc-password', 'path-password');
    bindVectorEyeToggle('toggle-mpin', 'mpin', 'path-mpin');
    bindVectorEyeToggle('toggle-totp', 'totp-secret', 'path-totp');
    bindVectorEyeToggle('toggle-gemini', 'gemini-api-key', 'path-gemini');

    // --- 2. Deterministic Broker Parameter Visibility Routing Engine ---
    const brokerSelect = document.getElementById('broker-select');
    const rowApiKey = document.getElementById('row-api-key');
    const rowSecretKey = document.getElementById('row-secret-key');
    const rowPassword = document.getElementById('row-password');
    const rowMpin = document.getElementById('row-mpin');
    const rowTotp = document.getElementById('row-totp-secret');

    if (brokerSelect) {
        brokerSelect.addEventListener('change', (e) => {
            const activeBroker = e.target.value;
            if (activeBroker === 'AngelOne') {
                if (rowApiKey) rowApiKey.style.display = 'block';
                if (rowSecretKey) rowSecretKey.style.display = 'none';
                if (rowPassword) rowPassword.style.display = 'none';
                if (rowMpin) rowMpin.style.display = 'block';
                if (rowTotp) rowTotp.style.display = 'block';
            } else if (activeBroker === 'Zerodha' || activeBroker === 'Sharekhan') {
                if (rowApiKey) rowApiKey.style.display = 'block';
                if (rowSecretKey) rowSecretKey.style.display = 'block';
                if (rowPassword) rowPassword.style.display = 'block';
                if (rowMpin) rowMpin.style.display = 'none';
                if (rowTotp) rowTotp.style.display = 'block';
            }
        });
    }

    // --- 3. Premium Gemini AI Toggles ---
    const aiToggle = document.getElementById('ai-toggle');
    const aiConfigPanel = document.getElementById('gemini-ai-config-panel');
    if (aiToggle && aiConfigPanel) {
        aiToggle.addEventListener('change', (e) => {
            aiConfigPanel.style.display = e.target.checked ? 'block' : 'none';
        });
    }

    // --- 4. Custom Chevron Profile Dropdown List Engine ---
    const listTrigger = document.getElementById('trigger-profile-list');
    const clientInput = document.getElementById('profile-client-input');
    const customDropdown = document.getElementById('custom-profile-dropdown');

    if (listTrigger && customDropdown) {
        listTrigger.addEventListener('click', (e) => {
            e.preventDefault();
            if (customDropdown.style.display === 'block') {
                customDropdown.style.display = 'none';
                return;
            }

            if (window.__TAURI__) {
                window.__TAURI__.core.invoke('get_all_saved_profiles')
                    .then((profiles) => {
                        customDropdown.innerHTML = '';
                        if (!profiles || profiles.length === 0) {
                            customDropdown.innerHTML = '<div style="padding: 8px; color: #777; font-size: 0.8rem; text-align: center;">No profiles saved yet</div>';
                            customDropdown.style.display = 'block';
                            return;
                        }

                        profiles.forEach(p => {
                            const row = document.createElement('div');
                            row.innerText = `${p.client_id} (${p.broker_type})`;
                            row.style.padding = '8px 12px';
                            row.style.cursor = 'pointer';
                            row.style.color = '#d1d4dc';
                            row.style.borderBottom = '1px solid #2b2b43';
                            row.addEventListener('mouseenter', () => { row.style.backgroundColor = '#004d4d'; });
                            row.addEventListener('mouseleave', () => { row.style.backgroundColor = '#1c1c1c'; });
                            row.addEventListener('click', () => {
                                if (clientInput) clientInput.value = p.client_id;
                                document.getElementById('api-key').value = p.api_key || "";
                                document.getElementById('secret-key').value = p.secret_key || "";
                                document.getElementById('acc-password').value = p.acc_password || "";
                                document.getElementById('mpin').value = p.mpin || "";
                                document.getElementById('totp-secret').value = p.totp_secret || "";
                                if (brokerSelect) {
                                    brokerSelect.value = p.broker_type;
                                    brokerSelect.dispatchEvent(new Event('change'));
                                }
                                customDropdown.style.display = 'none';
                            });
                            customDropdown.appendChild(row);
                        });
                        customDropdown.style.display = 'block';
                    });
            }
        });
        document.addEventListener('click', (e) => {
            if (e.target !== listTrigger && e.target !== clientInput && !customDropdown.contains(e.target)) {
                customDropdown.style.display = 'none';
            }
        });
    }

    // --- 5. Strict Session Authentication Gatekeeper ---
    const authBtn = document.getElementById('save-auth-btn');
    const rootOverlay = document.getElementById('login-wizard-overlay');
    const workspaceGrid = document.getElementById('workspace-grid');

    if (authBtn) {
        authBtn.addEventListener('click', async (e) => {
            e.preventDefault();
            const payload = {
                broker_type: brokerSelect ? brokerSelect.value : 'AngelOne',
                client_id: clientInput ? clientInput.value.trim() : '',
                api_key: document.getElementById('api-key').value.trim(),
                mpin: document.getElementById('mpin').value.trim(),
                totp_secret: document.getElementById('totp-secret').value.trim(),
                secret_key: document.getElementById('secret-key').value.trim(),
                acc_password: document.getElementById('acc-password').value.trim()
            };

            if (!payload.client_id || payload.client_id === "garbage") {
                alert("Security Rejection:\nInvalid or unpopulated Client ID vector parameters.");
                return;
            }

            if (window.__TAURI__) {
                const statusLabel = document.getElementById('api-status-text');
                if (statusLabel) statusLabel.innerText = "API Status: Authenticating...";
                try {
                    // 1. Commit profile parameters to hard disk config file first
                    await window.__TAURI__.core.invoke('save_trading_profile', payload);
                    
                    // 2. Commit profile secret to secure vault
                    await window.__TAURI__.core.invoke('save_secure_token', { clientId: payload.client_id, secret: JSON.stringify(payload) });

                    // 3. Set active profile
                    await window.__TAURI__.core.invoke('save_secure_token', { clientId: "active_client_id", secret: payload.client_id });
                    
                    // 4. Run verbose initialization handshake over the network bridge
                    const result = await window.__TAURI__.core.invoke('initialize_system_login', { brokerType: payload.broker_type });
                    
                    if (result.status === "SESSION_SUCCESS") {
                        if (statusLabel) statusLabel.innerText = "API Status: Live Token Connected";
                        if (rootOverlay) rootOverlay.style.setProperty('display', 'none', 'important');
                        if (workspaceGrid) workspaceGrid.style.setProperty('display', 'grid', 'important');
                        initializeChartEngine();
                    } else {
                        if (statusLabel) statusLabel.innerText = `API Status: Auth Failed (${result.error_message})`;
                        alert(`Access Denied:\n${result.error_message}`);
                    }
                } catch (err) {
                    console.error("Authentication Gate Rejection:", err);
                    if (statusLabel) statusLabel.innerText = `API Status: Auth Failed (${err})`;
                    alert(`Access Denied:\n${err}`);
                }
            }
        });
    }

    function initializeChartEngine() {
        const chartContainer = document.getElementById('tv-chart-container');
        if (!chartContainer || chartContainer.children.length > 0 || typeof LightweightCharts === 'undefined') return;
        try {
            const chart = LightweightCharts.createChart(chartContainer, {
                width: chartContainer.clientWidth,
                height: 450,
                layout: { background: { color: '#121212' }, textColor: '#d1d4dc' },
                grid: { vertLines: { color: '#1f2226' }, horzLines: { color: '#1f2226' } },
                timeScale: { timeVisible: true, secondsVisible: true }
            });
            const candleSeries = chart.addCandlestickSeries({ upColor: '#005a36', downColor: '#ef5350' });
            window.__TAURI__.event.listen('mock-market-tick', (event) => {
                const tick = event.payload;
                candleSeries.update({
                    time: Math.floor(Date.now() / 1000),
                    open: tick.open, high: tick.high, low: tick.low, close: tick.close
                });
            });
            window.addEventListener('resize', () => chart.resize(chartContainer.clientWidth, 450));
        } catch (e) {
            console.error("Chart build exception caught:", e);
        }
    }
});
