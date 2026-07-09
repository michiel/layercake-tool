import { useEffect } from 'react'

/**
 * Warn the user before leaving/reloading the tab while there are unsaved edits.
 *
 * Pass `dirty = true` whenever an editor holds local changes that have not been
 * persisted. This installs a `beforeunload` handler that triggers the browser's
 * native "Leave site?" confirmation, so closing the tab or hitting reload does
 * not silently discard in-progress work.
 *
 * Note: this only covers full-page navigation (tab close / reload / URL change).
 * In-app route changes are handled by the router, not here.
 */
export function useUnsavedChangesWarning(dirty: boolean): void {
  useEffect(() => {
    if (!dirty) {
      return
    }

    const handler = (event: BeforeUnloadEvent) => {
      event.preventDefault()
      // Required for the prompt to show in some browsers.
      event.returnValue = ''
      return ''
    }

    window.addEventListener('beforeunload', handler)
    return () => window.removeEventListener('beforeunload', handler)
  }, [dirty])
}
