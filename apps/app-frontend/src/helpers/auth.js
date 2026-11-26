/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank instance object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/core'

let pendingAccountSelection

// Example function:
// User goes to auth_url to complete flow, and when completed, authenticate_await_completion() returns the credentials
// export async function authenticate() {
//   const auth_url = await authenticate_begin_flow()
//   console.log(auth_url)
//   await authenticate_await_completion()
// }

/**
 * Check if the authentication servers are reachable, throwing an exception if
 * not reachable.
 */
export async function check_reachable() {
	await invoke('plugin:auth|check_reachable')
}

/**
 * Authenticate a user with Hydra - part 1.
 * This begins the authentication flow quasi-synchronously.
 *
 * @returns {Promise<DeviceLoginSuccess>} A DeviceLoginSuccess object with two relevant fields:
 * @property {string} verification_uri - The URL to go to complete the flow.
 * @property {string} user_code - The code to enter on the verification_uri page.
 */
export async function login() {
	return await invoke('plugin:auth|login')
}

export async function login_offline(username) {
	return await invoke('plugin:auth|login_offline', { username })
}

/**
 * Retrieves the default user
 * @return {Promise<UUID | undefined>}
 */
export async function get_default_user() {
	return await invoke('plugin:auth|get_default_user')
}

/**
 * Updates the default user
 * @param {UUID} user
 */
export async function set_default_user(user) {
	const previous = pendingAccountSelection
	const selection = (async () => {
		await previous?.catch(() => {})
		return await invoke('plugin:auth|set_default_user', { user })
	})()
	pendingAccountSelection = selection
	try {
		const result = await selection
		if (pendingAccountSelection === selection) {
			window.dispatchEvent(new Event('migurinth:account-changed'))
		}
		return result
	} finally {
		if (pendingAccountSelection === selection) pendingAccountSelection = undefined
	}
}

// A Play click immediately after a sidebar selection must not launch the old account.
export async function wait_for_account_selection() {
	while (pendingAccountSelection) {
		await pendingAccountSelection
	}
}

/**
 * Remove a user account from the database
 * @param {UUID} user
 */
export async function remove_user(user) {
	return await invoke('plugin:auth|remove_user', { user })
}

/**
 * Returns a list of users
 * @returns {Promise<Credential[]>}
 */
export async function users() {
	return await invoke('plugin:auth|get_users')
}
