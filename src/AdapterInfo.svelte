<script>
    export let adapterInfo;
</script>

<style>
    .content {
        margin-bottom: 20px;
    }
    
    .indented {
        margin-left: 20px;
    }
</style>

<div class="content">
    <div>
        <b>Name:</b> 
        {adapterInfo.name}
    </div>
    <div>
        <b>Description:</b> 
        {adapterInfo.description}
    </div>
    <div>
        <b>Combo Index:</b> 
        {adapterInfo.comboIndex}
    </div>
    <div>
        <b>Hardware Address: </b>
        {adapterInfo.address.map((n) => n.toString(16).toUpperCase().padStart(2, '0')).join('')}
    </div>
    <div>
        <div>
            <b>Ip Addresses</b>
        </div>
        {#each adapterInfo.ipAddresses as addr}
            <div class = "indented">
                <div>
                    <b>Address: </b> 
                    {addr.address}
                </div>
                <div>
                    <b>Mask: </b>
                    {addr.mask}
                </div>
            </div>
        {/each}
    </div>
    <div>
        <div>
            <b>Gateways</b>
        </div>
        {#each adapterInfo.gateways as addr}
            <div class = "indented">
                <div>
                    <b>Address: </b> 
                    {addr.address}
                </div>
                <div>
                    <b>Mask: </b>
                    {addr.mask}
                </div>
            </div>
        {/each}
    </div>
</div>