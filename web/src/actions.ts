import type { Action } from './contract'

/**
 * `url` if it is a web link, else undefined. Module-supplied links pass through
 * this both as a link's href (mouse) and before opening one (Enter), so a
 * module can't make either open something else: a `file:` URL, `javascript:`,
 * or an app's custom scheme.
 */
export function webUrl(url: string | null | undefined): string | undefined {
  return url && /^https?:\/\//i.test(url) ? url : undefined
}

/**
 * Carries out an action: an item's, from Enter (a click on an item's link
 * uses the link itself), or a click on a link in markdown text. New kinds of
 * action are added here.
 */
export function runAction(action: Action) {
  switch (action.type) {
    case 'open_url':
      if (webUrl(action.url)) window.open(action.url, '_blank', 'noopener')
      break
    case 'run_command':
      // Not run yet; `sitroom plugin-check` says so.
      break
  }
}
