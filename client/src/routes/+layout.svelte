<script lang="ts">
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import AppSidebar from '$lib/components/app-sidebar.svelte';
	import '../app.css';
	import favicon from '$lib/assets/favicon.ico';
	import { ModeWatcher } from 'mode-watcher';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<ModeWatcher />

<Sidebar.Provider>
	<AppSidebar />
	<main class="flex w-full flex-col space-y-2 p-3">
		<Sidebar.Trigger />
		{@render children?.()}
	</main>
</Sidebar.Provider>
