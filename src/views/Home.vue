<script setup>
const greetMsg = ref("");
const name = ref("");

async function greet() {
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <main class="bg-black/90 min-h-screen flex items-center justify-center">
  </main>
</template>