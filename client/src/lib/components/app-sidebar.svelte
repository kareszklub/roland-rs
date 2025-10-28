<script lang="ts">
	import HouseIcon from '@lucide/svelte/icons/house';
	import XIcon from '@lucide/svelte/icons/x';
	import CheckIcon from '@lucide/svelte/icons/check';
	import ScrollIcon from '@lucide/svelte/icons/scroll-text';
	import JoystickIcon from '@lucide/svelte/icons/joystick';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import ModeToggle from './mode-toggle.svelte';
	import Input from './ui/input/input.svelte';
	import Button from './ui/button/button.svelte';
	import * as Avatar from '$lib/components/ui/avatar/index.js';
	import { Spinner } from '$lib/components/ui/spinner/index.js';

	import favicon from '$lib/assets/favicon.ico';
	import { roland, ws_connect, ws_disconnect } from '$lib/ws.svelte.js';

	const items = [
		{
			title: 'Home',
			url: '/',
			icon: HouseIcon
		},
		{
			title: 'Logs',
			url: '/logs',
			icon: ScrollIcon
		},
		{
			title: 'Controller',
			url: '/controller',
			icon: JoystickIcon
		},
		{
			title: 'Settings',
			url: '/settings',
			icon: SettingsIcon
		}
	];
</script>

<Sidebar.Root>
	<Sidebar.Header>
		<div class="flex justify-between">
			<Avatar.Root>
				<Avatar.Image src={favicon} alt="Karesz" />
				<Avatar.Fallback>KK</Avatar.Fallback>
			</Avatar.Root>
			<ModeToggle />
		</div>
		<h1 class="font-bold">Roland Control Panel</h1>
		<Input bind:value={roland.ip}></Input>
		{#if roland.connection === 'disconnected'}
			<Button onclick={ws_connect}>Connect</Button>
		{:else if roland.connection === 'connected'}
			<Button onclick={ws_disconnect}>Disconnect</Button>
		{:else}
			<Button disabled>Connect</Button>
		{/if}
	</Sidebar.Header>
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Status</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					<Sidebar.MenuItem>
						<Sidebar.MenuButton>
							{#if roland.connection === 'disconnected'}
								<XIcon class="text-red-500" />
								<span class="text-red-400">Disconnected</span>
							{:else if roland.connection === 'connected'}
								<CheckIcon class="text-green-500" />
								<span class="text-green-400">Connected</span>
							{:else}
								<Spinner />
								Connecting
							{/if}
						</Sidebar.MenuButton>
					</Sidebar.MenuItem>
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
		<Sidebar.Group>
			<Sidebar.GroupLabel>Application</Sidebar.GroupLabel>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each items as item (item.title)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton>
								{#snippet child({ props })}
									<a href={item.url} {...props}>
										<item.icon />
										<span>{item.title}</span>
									</a>
								{/snippet}
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
</Sidebar.Root>
