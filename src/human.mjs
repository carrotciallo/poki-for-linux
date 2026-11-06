import { platform } from 'node:os';
import { execute } from './work.mjs';

// A real macOS dialog is the acknowledgment. A successful notification alone is not.
export async function askHuman(message, seconds = 120) {
    if (platform() === 'linux') return askHumanLinux(message, seconds);
    const script = `on run argv
      set promptText to item 1 of argv
      set waitSeconds to (item 2 of argv) as integer
      try
        set response to display dialog promptText with title "Poki MCP" buttons {"取消", "确认"} default button "确认" cancel button "取消" giving up after waitSeconds
        if gave up of response then return "timeout"
        if button returned of response is "确认" then return "done"
        return "cancelled"
      on error number -128
        return "cancelled"
      end try
    end run`;
    const { stdout } = await execute('osascript', ['-e', script, '--', message, String(Math.ceil(seconds))], { timeout: (seconds + 10) * 1000 });
    const status = stdout.trim();
    return { status, completed: status === 'done', message: status === 'done' ? 'The user confirmed the dialog. Inspect the device before continuing.' : 'The user did not confirm completion.' };
}

// The Linux counterpart of the macOS dialog: zenity when a desktop is
// available. Without one there is no human channel, so fail loudly instead
// of pretending the prompt happened.
async function askHumanLinux(message, seconds) {
    try {
        await execute('zenity', ['--question', '--title=Poki MCP', `--text=${message}`, '--ok-label=确认', '--cancel-label=取消', `--timeout=${Math.ceil(seconds)}`], { timeout: (seconds + 10) * 1000 });
        return { status: 'done', completed: true, message: 'The user confirmed the dialog. Inspect the device before continuing.' };
    } catch (error) {
        if (error.code === 'ENOENT') throw new Error('No approval dialog is available on this Linux host because zenity is not installed. Install zenity, or set POKI_MCP_APPROVAL_MODE=allow only if unapproved sensitive actions are acceptable here.');
        const status = error.code === 5 ? 'timeout' : 'cancelled';
        return { status, completed: false, message: 'The user did not confirm completion.' };
    }
}
