<script>
    import KeyValue from './KeyValue.svelte';

    export let adapterInfo;
</script>

<style>
    .content {
        margin-bottom: 20px;
    }
    
    .indented {
        margin-left: 20px;
    }
    
    .bold {
        font-weight: bold;
    }
</style>

<div class="content">
    <KeyValue key="Name" bind:value={adapterInfo.name} />
    <KeyValue key="Description" bind:value={adapterInfo.description} />
    <KeyValue key="Combo Index" bind:value={adapterInfo.comboIndex} />
    <!-- TODO: Make this value part of AdapterInfo -->
    <KeyValue key="Hardware Address" value={adapterInfo.address.map((n) => n.toString(16).toUpperCase().padStart(2, '0')).join('')} />
    <div>
        <div>
            <span class="bold">Ip Addresses</span>
        </div>
        {#each adapterInfo.ipAddresses as addr}
            <div class = "indented">
                <KeyValue key="Address" bind:value={addr.address} />
                <KeyValue key="Mask" bind:value={addr.mask} />
            </div>
        {/each}
    </div>
    <div>
        <div>
            <span class="bold">Gateways</span>
        </div>
        {#each adapterInfo.gateways as addr}
            <div class = "indented">
                <KeyValue key="Address" bind:value={addr.address} />
                <KeyValue key="Mask" bind:value={addr.mask} />
            </div>
        {/each}
    </div>
</div>