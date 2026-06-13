<script lang="ts">
	import { goto } from '$app/navigation';
	import { ProjectsHome } from '@glyph/ui/application';
	import { projects } from '@glyph/ui/projects';
	import { projectHost } from '$lib/project';

	/** Open a disk-backed project folder: remember it, then route to the editor. */
	async function openFolder() {
		const root = await projectHost.pickFolder('Open project folder');
		if (!root) return;
		const p = projects.remember(root);
		goto(`/editor/${p.id}`);
	}

	/** Import a .zip → extracted folder → remember → editor. */
	async function importZip() {
		const zip = await projectHost.pickImportFile();
		if (!zip) return;
		const root = await projectHost.importZip(zip);
		if (!root) return;
		const p = projects.remember(root);
		goto(`/editor/${p.id}`);
	}
</script>

<svelte:head>
	<title>Glyph — Projects</title>
</svelte:head>

<ProjectsHome
	projects={projects.list}
	oncreate={() => {
		const p = projects.create();
		goto(`/editor/${p.id}`);
	}}
	onopenfolder={openFolder}
	onimport={importZip}
	onopen={(id) => {
		projects.touch(id);
		goto(`/editor/${id}`);
	}}
	onrename={(id, name) => projects.rename(id, name)}
	onduplicate={(id) => projects.duplicate(id)}
	ondelete={(id) => projects.remove(id)}
	onsettings={() => goto('/settings')}
/>
