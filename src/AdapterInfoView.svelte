<script>
    import { Accordion, AccordionItem } from "carbon-components-svelte";
    import AdapterInfo from './AdapterInfo.svelte';
    
    export let adapterInfo;
</script>

<style>

</style>

<div>
    <h2>Adapters</h2>
    {#await adapterInfo}
        <div>Loading...</div>
        {:then adapterInfo}
            {#each adapterInfo as adapter}
                <Accordion>
                    <!-- TODO: Consider making part of AdapterInfo-->
                    <AccordionItem>
                        <div slot="title">
                            <h5>{adapter.name}</h5>
                            <div>{adapter.description}</div>
                        </div>
                        <AdapterInfo bind:adapterInfo={adapter}/>
                    </AccordionItem>
                </Accordion>
            {/each}
        {:catch error}
            <div> Error: {error} </div>
    {/await}
</div>