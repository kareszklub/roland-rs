<script lang="ts">
	import { logs, append_log, LogLevel } from '$lib/logs.svelte.js';
</script>

<div class="flex flex-col">
	{#if logs.length > 0}
		{#each logs as log}
			<div>
				<span class="text-primary/70">{log.time}</span>
				<span class={['inline-block w-13', log.level_style]}>{log.level}</span>
				<span>{log.text}</span>
			</div>
		{/each}
	{:else}
		<p class="text-primary/70">No logs yet...</p>
	{/if}
</div>
