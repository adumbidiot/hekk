<script>
    import * as tauri from 'tauri/api/tauri';
    import * as tauriEvent from 'tauri/api/event';
    import { Tabs, Tab, TabContent } from "carbon-components-svelte";
    import AdapterInfoView from './AdapterInfoView.svelte';
    
    async function getVersion() {
        let version = await tauri.promisified({ cmd: 'Version' });
        return version;
    }
    
    async function getAdapterInfo() {
        let info = await tauri.promisified({ cmd: 'GetAdapterInfo' });
        return info;
    }
    
    export let adapterInfo = getAdapterInfo();
    
    getVersion().then(console.log);
    
    window.onTauriInit = function() {
        tauriEvent.emit('GetVersion');
        tauriEvent.listen('SetVersion', (version) => {
            console.log("Event Version:", version); 
        });
    }
</script>

<style lang="scss" global>
    @import "carbon-components-svelte/css/g90";
</style>

<main>
    <Tabs>
      <Tab label="Adapters" />
      <Tab label="Tab label 2" />
      <Tab label="Tab label 3" />
      <div slot="content">
        <TabContent>
            <AdapterInfoView bind:adapterInfo />
        </TabContent>
        <TabContent>Content 2</TabContent>
        <TabContent>Content 3</TabContent>
      </div>
    </Tabs>
</main>