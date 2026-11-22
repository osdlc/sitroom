<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import type { ClientMessage, NewDashboardFrom, UiSettings } from '../contract'

const props = defineProps<{ settings: UiSettings; from: NewDashboardFrom; before: number }>()
const emit = defineEmits<{ close: []; created: [blank: boolean]; send: [msg: ClientMessage] }>()

const name = ref('')
const from = ref<NewDashboardFrom>(props.from)
const waiting = ref(false)
const input = ref<HTMLInputElement>()
onMounted(() => input.value?.focus())

/** This dialog's request: outcomes are counted, and the last one stays in the settings. */
const outcome = computed(() => {
  const created = props.settings.config.created
  return created && (created.seq ?? 0) > props.before ? created : null
})
watch(outcome, (o) => {
  if (!o) return
  waiting.value = false
  if (!o.error) emit('created', from.value === 'blank')
})

function submit() {
  if (waiting.value || !name.value.trim()) return
  waiting.value = true
  emit('send', { type: 'create_dashboard', name: name.value, from: from.value })
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')" @keydown.esc="emit('close')">
    <section class="help new-dashboard" role="dialog" aria-label="New dashboard">
      <header>
        <h2>{{ props.from === 'copy' ? 'Save dashboard as' : 'New dashboard' }}</h2>
        <button type="button" @click="emit('close')">Close</button>
      </header>
      <form @submit.prevent="submit">
        <label>
          Name
          <input ref="input" v-model="name" type="text" autocomplete="off" spellcheck="false" />
        </label>
        <div class="choices">
          <label><input v-model="from" type="radio" value="starter" /> Starter template</label>
          <label><input v-model="from" type="radio" value="blank" /> Blank</label>
          <label><input v-model="from" type="radio" value="copy" /> Copy of current</label>
        </div>
        <p class="muted">
          {{
            from === 'copy'
              ? `A copy of ${settings.config.dashboard ?? 'config.yaml'}, as its own file you can change.`
              : from === 'starter'
                ? 'Ready-made screens to start from: change, remove or add tiles in the editor.'
                : 'One empty screen. Modules in config.yaml keep running; add what you want to show.'
          }}
        </p>
        <p v-if="outcome?.error" class="error" role="alert">{{ outcome.error }}</p>
        <button type="submit" :disabled="waiting || !name.trim()">Create</button>
      </form>
    </section>
  </div>
</template>
