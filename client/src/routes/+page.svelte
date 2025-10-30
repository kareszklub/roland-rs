<script lang="ts">
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';
	import JoystickIcon from '@lucide/svelte/icons/joystick';
	import RouteIcon from '@lucide/svelte/icons/route';
	import ArrowLeftToLine from '@lucide/svelte/icons/arrow-left-to-line';
	import { roland_state, handle_control_state } from './controller/controller.svelte';
	import { buttonVariants } from '$lib/components/ui/button';
	import * as Tabs from '$lib/components/ui/tabs/index.js';
</script>

<div class="flex flex-col items-center space-y-4">
	<div class="mb-8">
		<Tabs.Root bind:value={roland_state.control_state}>
			<Tabs.List>
				<Tabs.Trigger value="FollowLine"><RouteIcon />Follow line</Tabs.Trigger>
				<Tabs.Trigger value="ManualControl"><JoystickIcon />Manual control</Tabs.Trigger>
				<Tabs.Trigger value="KeepDistance"><ArrowLeftToLine />Keep distance</Tabs.Trigger>
			</Tabs.List>
		</Tabs.Root>
	</div>

	<div class="flex items-center space-x-2">
		<p class="text-primary/70">Distance:</p>
		<div class={buttonVariants({ variant: 'outline' })}>
			{#if roland_state.ultra_sensor === null}
				--
			{:else}
				{String(roland_state.ultra_sensor).padStart(2, '0')}
			{/if}
			<span>cm</span>
		</div>
	</div>

	<div class="flex space-x-1">
		{#if roland_state.track_sensor}
			{#each roland_state.track_sensor as off}
				<div
					class="{buttonVariants({ variant: 'outline' })} h-13 w-13"
					style="background-color: {off ? 'white' : 'blue'};"
				></div>
			{/each}
		{:else}
			{#each new Array(4) as _}
				<div class="{buttonVariants({ variant: 'outline' })} h-13 w-13"></div>
			{/each}
		{/if}
	</div>
</div>
