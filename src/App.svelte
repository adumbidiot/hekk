<script>
    import * as tauri from 'tauri/api/tauri';
    import * as tauriEvent from 'tauri/api/event';
    import { Tabs, Tab, TabContent } from "carbon-components-svelte";
    import AdapterInfo from './AdapterInfo.svelte';
    
    async function getVersion() {
        let version = await tauri.promisified({ cmd: 'Version' });
        return version;
    }
    
    async function getAdapterInfo() {
        let info = await tauri.promisified({ cmd: 'GetAdapterInfo' });
        return info;
    }
    
    export let name;
    
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
    @import "carbon-components-svelte/css/g10";
    main {
        margin: 0 auto;
    }
</style>
<!--<style>
	h1 {
		color: #ff3e00;
		text-transform: uppercase;
		font-size: 4em;
		font-weight: 100;
	}
	@media (min-width: 640px) {
		main {
			max-width: none;
		}
	}
    
    .adapter-info-wrapper {
        text-align: left;
    }
</style>-->

<main>
    <!--
	<h1>Hello {name}!</h1>
	<p>Visit the <a href="https://svelte.dev/tutorial" target="_blank">Svelte tutorial</a> to learn how to build Svelte apps.</p>
    -->
    <Tabs>
      <Tab label="Tab label 1" />
      <Tab label="Tab label 2" />
      <Tab label="Tab label 3" />
      <div slot="content">
        <TabContent>Content 1</TabContent>
        <TabContent>Content 2</TabContent>
        <TabContent>Content 3</TabContent>
      </div>
    </Tabs>
    <div class="adapter-info-wrapper" >
        <h2>Adapters</h2>
        {#await adapterInfo}
            {:then adapterInfo}
                {#each adapterInfo as adapter}
                    <AdapterInfo bind:adapterInfo={adapter}/>
                {/each}
            {:catch error}
                <div> Error: {error} </div>
        {/await}
    </div>
</main>