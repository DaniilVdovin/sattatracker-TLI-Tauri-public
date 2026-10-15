<template>
  <DraggableWidget
    :widgetId="widgetId"
    :title="title"
    :defaultPosition="defaultPosition"
    :alwaysInteractive="clickable"
  >
    <div
      class="flex flex-col justify-center items-center pointer-events-none w-full h-full text-center overflow-hidden"
      :class="{ 'cursor-pointer !pointer-events-auto': clickable }"
      style="font-family: var(--theme-font-main); min-width: 80px;"
      @click="clickable ? emit('widgetClick') : undefined"
    >
      <div
        class="font-bold uppercase tracking-widest mb-[2px] select-none whitespace-nowrap overflow-hidden text-ellipsis w-full"
        style="color: var(--theme-text-qty); font-size: clamp(8px, min(calc(var(--widget-w, 110) * 0.09px), calc(var(--widget-h, 60) * 0.15px)), 32px); line-height: 1.1;"
      >{{ title }}</div>
      <div
        class="font-bold w-full whitespace-nowrap overflow-hidden text-ellipsis"
        :style="{
          color: isNegative ? '#ff5555' : 'var(--theme-text-price)',
          fontSize: 'clamp(10px, min(calc(var(--widget-w, 110) * 0.14px), calc(var(--widget-h, 60) * 0.28px)), 64px)',
          lineHeight: '1.2'
        }"
      >
        {{ isPositive && typeof value === 'number' && value > 0 ? '+' : '' }}{{ fmtValue }}{{ unit }}
      </div>
    </div>
  </DraggableWidget>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import DraggableWidget from './DraggableWidget.vue';

const props = defineProps<{
  widgetId: string;
  title: string;
  value: string | number;
  unit: string;
  isPositive?: boolean;
  clickable?: boolean;
  defaultPosition?: { x: number, y: number };
}>();

const emit = defineEmits<{ (e: 'widgetClick'): void }>();

const isNegative = computed(() => {
  if (typeof props.value === 'number') return props.value < 0;
  if (typeof props.value === 'string' && props.value.startsWith('-')) return true;
  return false;
});

const fmtValue = computed(() => {
  if (typeof props.value !== 'number') return props.value;
  return parseFloat(props.value.toFixed(2));
});
</script>
