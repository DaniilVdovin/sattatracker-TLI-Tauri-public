<template>
  <div class="setting-row">
    <div class="setting-name text-sm" style="min-width:110px;">{{ label }}</div>
    <div class="flex items-center gap-2 shrink-0">
      <input
        type="color"
        :value="hex"
        @input="onHex"
        class="rounded cursor-pointer shrink-0"
        style="width:28px;height:24px;padding:1px;border:1px solid rgba(255,255,255,0.15);background:transparent;"
      />
      <template v-if="showAlpha">
        <input
          type="range"
          :value="alpha"
          min="0" max="1" step="0.05"
          class="w-20 accent-blue-500"
          @input="onAlpha"
        />
        <span class="text-xs font-mono w-8 text-right" style="color:rgba(255,255,255,0.4);">
          {{ Math.round(alpha * 100) }}%
        </span>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  label: string;
  hex: string;
  alpha: number;
  showAlpha?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:hex',   v: string): void;
  (e: 'update:alpha', v: number): void;
  (e: 'update'): void;
}>();

function onHex(e: Event) {
  emit('update:hex', (e.target as HTMLInputElement).value);
  emit('update');
}
function onAlpha(e: Event) {
  emit('update:alpha', parseFloat((e.target as HTMLInputElement).value));
  emit('update');
}
</script>
