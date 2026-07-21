use gpui::{AppContext, AsyncApp, Global};

#[derive(Clone)]
pub struct WifiManager {
    nm: nmrs::NetworkManager,
    connect_tx: smol::channel::Sender<(nmrs::Network, nmrs::WifiSecurity)>,
}

impl WifiManager {
    pub async fn new(cx: &mut AsyncApp) -> nmrs::Result<Self> {
        let nm = nmrs::NetworkManager::new().await?;
        let (tx, rx) = smol::channel::bounded::<(nmrs::Network, nmrs::WifiSecurity)>(1);

        cx.background_spawn({
            let nm = nm.clone();
            async move {
                while let Ok((network, security)) = rx.recv().await {
                    nm.connect(&network.ssid, None, security).await.ok();
                }
            }
        })
        .detach();

        Ok(Self { nm, connect_tx: tx })
    }

    pub fn enabled(&self) -> bool {
        let state = smol::block_on(self.nm.wifi_state()).unwrap();
        state.enabled || state.hardware_enabled
    }

    pub fn enable(&self, state: bool) -> nmrs::Result<()> {
        smol::block_on(self.nm.set_wireless_enabled(state))
    }

    pub fn list(&self) -> Vec<nmrs::Network> {
        smol::block_on(self.nm.list_networks(None)).unwrap_or_default()
    }

    pub fn connect(&self, network: &nmrs::Network, security: nmrs::WifiSecurity) {
        self.connect_tx
            .try_send((network.clone(), security))
            .unwrap();
    }
}

impl Global for WifiManager {}
