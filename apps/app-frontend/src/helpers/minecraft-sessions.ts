import { get_default_user } from '@/helpers/auth'
import { get_by_instance_id, kill } from '@/helpers/process'

export type MinecraftSessionProcess = {
	uuid: string
	instance_id: string
	account_id: string
}

export async function getCurrentAccountProcess(
	instanceId: string,
): Promise<MinecraftSessionProcess | null> {
	const [selectedAccount, processes] = await Promise.all([
		get_default_user(),
		get_by_instance_id(instanceId),
	])
	if (typeof selectedAccount !== 'string' || !Array.isArray(processes)) return null
	return (
		processes.find(
			(process): process is MinecraftSessionProcess =>
				process !== null &&
				typeof process === 'object' &&
				process.account_id === selectedAccount &&
				typeof process.uuid === 'string',
		) ?? null
	)
}

export async function stopCurrentAccountProcess(instanceId: string): Promise<boolean> {
	const process = await getCurrentAccountProcess(instanceId)
	if (!process) return false
	await kill(process.uuid)
	return true
}
