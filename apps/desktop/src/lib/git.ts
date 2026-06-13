import { invoke } from '@tauri-apps/api/core';
import type { GitProvider } from '@glyphx/ui/application';

/**
 * Desktop Git provider — local version control via the Rust `git` module
 * (gitoxide). Operates on the open project folder; pure-Rust so it needs no
 * system `git` install.
 */
export const gitProvider: GitProvider = {
	isRepo: (root) => invoke('git_is_repo', { root }),
	init: (root) => invoke('git_init', { root }),
	head: (root) => invoke('git_head', { root }),
	status: (root) => invoke('git_status', { root }),
	stage: (root, paths) => invoke('git_stage', { root, paths }),
	unstage: (root, paths) => invoke('git_unstage', { root, paths }),
	diff: (root, path, staged) => invoke('git_diff', { root, path, staged }),
	commit: (root, message) => invoke('git_commit', { root, message }),
	log: (root, limit) => invoke('git_log', { root, limit }),
	clone: (url, dest) => invoke('git_clone', { url, dest }),
	remotes: (root) => invoke('git_remotes', { root }),
	fetch: (root, url) => invoke('git_fetch', { root, url }),
	pull: (root, url) => invoke('git_pull', { root, url }),
	push: (root, url, branch) => invoke('git_push', { root, url, branch })
};
