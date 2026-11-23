<script setup lang="ts">
// Markdown in a text widget (`markdown: true`), rendered with
// @osdlc/markdown-render: CommonMark, GFM and Obsidian's additions. The TUI
// and the native app read the same text with comrak
// (crates/core/src/markdown.rs). As there, a single line break is kept (the
// `white-space` rule in style.css), and `%% comments %%` are left as text:
// a text widget is a module's output, not a vault note, and doubled percent
// signs in it are not comments. HTML tags are dropped by the renderer's
// sanitizer, and the comrak side hides them to match. Links open in a new
// tab, through runAction.
//
// The renderer works asynchronously, so the text takes its room after the
// tile has been fitted. Each finished render sends a bubbling `tile-refit`
// event, and TileCard fits the tile again.
import { MarkdownRender } from '@osdlc/markdown-render-vue'
import '@osdlc/markdown-render/theme.css'
import { ref } from 'vue'
import { runAction } from '../actions'

defineProps<{ source: string }>()

const options = { comments: false }
// A tile is small: plain code blocks and tables, without copy and sort chrome.
const components = { pre: undefined, table: undefined }

const root = ref<HTMLElement>()
function rendered() {
  root.value?.dispatchEvent(new Event('tile-refit', { bubbles: true }))
}

function click(event: MouseEvent) {
  const link = (event.target as HTMLElement | null)?.closest('a')
  if (!link) return
  event.preventDefault()
  runAction({ type: 'open_url', url: link.getAttribute('href') ?? '' })
}
</script>

<template>
  <div ref="root" class="w-md" data-widget="text" @click="click">
    <MarkdownRender
      :source="source"
      :options="options"
      :components="components"
      @rendered="rendered"
    />
  </div>
</template>
