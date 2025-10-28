<script lang="ts">
	import {
		keys_down,
		on_key_change,
		roland_state,
		handle_servo,
		handle_buzzer,
		handle_led
	} from './controller.svelte';
	import Button, { buttonVariants } from '$lib/components/ui/button/button.svelte';
	import { Slider } from '$lib/components/ui/slider/index.js';
	import MegaphoneIcon from '@lucide/svelte/icons/megaphone';
	import MegaphoneOffIcon from '@lucide/svelte/icons/megaphone-off';
	import { Input } from '$lib/components/ui/input';
	import LightbulbIcon from '@lucide/svelte/icons/lightbulb';
	import LightbulbOffIcon from '@lucide/svelte/icons/lightbulb-off';
	import * as Dialog from '$lib/components/ui/dialog/index.js';

	const on_key_down = (e: KeyboardEvent) => {
		if (e.repeat) return;
		keys_down.add(e.key);
	};

	const on_key_up = (e: KeyboardEvent) => {
		keys_down.delete(e.key);
	};

	let is_horn_pressed = $state(false);

	let led_color = $derived(
		`rgb(${roland_state.led.r}, ${roland_state.led.g}, ${roland_state.led.b})`
	);

	let led_color_accent = $derived(
		(() =>
			(roland_state.led.r + roland_state.led.g + roland_state.led.b) / 3 >= 128
				? 'black'
				: 'white')()
	);
</script>

<svelte:window
	on:keydown={(e: KeyboardEvent) => {
		on_key_down(e);
		on_key_change();
	}}
	on:keyup={(e: KeyboardEvent) => {
		on_key_up(e);
		on_key_change();
	}}
/>

<div class="flex justify-center">
	<div class="flex w-fit flex-col space-y-6">
		<!-- WASD -->
		<div class="grid grid-cols-3 grid-rows-2 gap-3">
			{#each ['', 'w', '', 'a', 's', 'd'] as letter}
				{#if letter !== ''}
					<Button
						variant="secondary"
						class="size-20 text-2xl"
						onmousedown={() => {
							keys_down.add(letter);
							on_key_change();
						}}
						onmouseup={() => {
							keys_down.delete(letter);
							on_key_change();
						}}
						ontouchstart={() => {
							keys_down.add(letter);
							on_key_change();
						}}
						ontouchend={(e) => {
							e.preventDefault();
							keys_down.delete(letter);
							on_key_change();
						}}
					>
						{letter.toUpperCase()}
					</Button>
				{:else}
					<div></div>
				{/if}
			{/each}
		</div>
		<!-- Speed control -->
		<div class="flex flex-col space-y-2">
			<div class="flex justify-between">
				<div class="text-primary/70">Speed:</div>
				<div><span>{Math.round(roland_state.speed_multiplier * 100)}</span>%</div>
				<div class="w-[56.3px]"></div>
			</div>
			<Slider type="single" bind:value={roland_state.speed_multiplier} max={1} step={0.01} />
		</div>
		<!-- Servo -->
		<div class="flex flex-col space-y-2">
			<div class="flex justify-between">
				<div class="text-primary/70">Servo:</div>
				<div><span>{roland_state.servo_angle - 90}</span>&deg;</div>
				<div class="w-[56.3px]"></div>
			</div>
			<Slider
				type="single"
				bind:value={roland_state.servo_angle}
				max={180}
				step={10}
				onValueCommit={handle_servo}
			/>
		</div>
		<div class="flex flex-col items-center space-y-2"></div>
		<!-- Buzzer -->
		<div class="flex space-x-2">
			<Button
				variant={is_horn_pressed ? 'destructive' : 'secondary'}
				onmousedown={() => {
					is_horn_pressed = true;
					handle_buzzer(true);
				}}
				onmouseup={() => {
					is_horn_pressed = false;
					handle_buzzer(false);
				}}
				ontouchstart={() => {
					is_horn_pressed = true;
					handle_buzzer(true);
				}}
				ontouchend={() => {
					is_horn_pressed = false;
					handle_buzzer(false);
				}}
				class="size-20"
			>
				{#if is_horn_pressed}
					<MegaphoneIcon class="size-xl" />
				{:else}
					<MegaphoneOffIcon class="size-xl" />
				{/if}
			</Button>
			<div class="flex flex-col justify-center">
				<Dialog.Root>
					<Dialog.Trigger class={buttonVariants({ variant: 'outline' })}>
						{roland_state.buzzer_freq} Hz
					</Dialog.Trigger>
					<Dialog.Content class="w-75">
						<Dialog.Header>
							<Dialog.Title>Set Buzzer frequency</Dialog.Title>
						</Dialog.Header>
						<Input bind:value={roland_state.buzzer_freq}></Input>
						<Slider
							type="single"
							bind:value={roland_state.buzzer_freq}
							min={20}
							max={5000}
							step={1}
						/>
					</Dialog.Content>
				</Dialog.Root>
			</div>
		</div>
		<!-- LED -->
		<div class="flex justify-center">
			<Dialog.Root>
				<Dialog.Trigger
					class={buttonVariants({ variant: 'outline' })}
					style="background-color: {led_color}; width: 80px; height: 80px;"
				>
					{#if roland_state.led.r + roland_state.led.g + roland_state.led.b === 0}
						<LightbulbOffIcon class="size-xl text-white" />
					{:else}
						<LightbulbIcon style="color: {led_color_accent};" class="size-xl" />
					{/if}
				</Dialog.Trigger>
				<Dialog.Content class="w-60">
					<Dialog.Header>
						<Dialog.Title>Set LED color</Dialog.Title>
					</Dialog.Header>
					<div class="flex space-x-2">
						<span>R:</span>
						<Slider
							type="single"
							bind:value={roland_state.led.r}
							slider_color="#ff0000"
							max={255}
							step={15}
							class="w-30"
							onValueCommit={handle_led}
						/>
					</div>
					<div class="flex space-x-2">
						<span>G:</span>
						<Slider
							type="single"
							bind:value={roland_state.led.g}
							slider_color="#00ff00"
							max={255}
							step={15}
							class="w-30"
							onValueCommit={handle_led}
						/>
					</div>
					<div class="flex space-x-2">
						<span>B:</span>
						<Slider
							type="single"
							slider_color="#0000ff"
							bind:value={roland_state.led.b}
							max={255}
							step={15}
							class="w-30"
							onValueCommit={handle_led}
						/>
					</div>
				</Dialog.Content>
			</Dialog.Root>
		</div>
	</div>
</div>
