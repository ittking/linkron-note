<script setup>
import { theme } from "ant-design-vue"

const themeConfig = {
  token: {
    colorPrimary: '#00ff88',
  },
  algorithm: theme.compactAlgorithm,
}
</script>

<template>
  <a-config-provider :theme="themeConfig">
    <router-view />
  </a-config-provider>
</template>
