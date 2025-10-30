<script lang="ts">
	import * as ToggleGroup from '$lib/components/ui/toggle-group/index.js';
	import JoystickIcon from '@lucide/svelte/icons/joystick';
	import RouteIcon from '@lucide/svelte/icons/route';
	import ArrowLeftToLine from '@lucide/svelte/icons/arrow-left-to-line';
	import { roland_state, handle_control_state } from './controller/controller.svelte';
	import { buttonVariants } from '$lib/components/ui/button';
	import { MediaQuery } from 'svelte/reactivity';
	import * as RadioGroup from '$lib/components/ui/radio-group/index.js';
	import { Label } from '$lib/components/ui/label';
</script>

<div class="flex flex-col items-center space-y-4">
	<div class="mb-8">
		<RadioGroup.Root bind:value={roland_state.control_state} onValueChange={handle_control_state}>
			<div class="flex items-center space-x-2">
				<RadioGroup.Item value="FollowLine" id="r1" />
				<Label for="r1">Follow line</Label>
			</div>
			<div class="flex items-center space-x-2">
				<RadioGroup.Item value="ManualControl" id="r2" />
				<Label for="r2">Manual control</Label>
			</div>
			<div class="flex items-center space-x-2">
				<RadioGroup.Item value="KeepDistance" id="r3" />
				<Label for="r3">Keep distance</Label>
			</div>
		</RadioGroup.Root>
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
		{#each roland_state.track_sensor as off}
			<div
				class="{buttonVariants({ variant: 'outline' })} h-13 w-13"
				style="background-color: {off ? 'white' : 'blue'};"
			></div>
		{/each}
	</div>
</div>
