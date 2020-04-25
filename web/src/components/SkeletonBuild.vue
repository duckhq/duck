<template>
  <div class="cursor-pointer text-gray-100 p-4 rounded-lg shadow-lg light">
    <div class="flex flex-col justify-between h-full">
      <!-- Project -->
      <div class="flex justify-between align-middle">
        <span class="dark rounded-md h-6 w-1/2"></span>
        <span class="dark rounded-md h-8 w-8"></span>
      </div>
      <div>
        <!-- Project -->
        <div class="my-3 flex flex-row justify-start medium rounded-md h-3 w-1/4">
          
        </div>

        <!-- Branch -->
        <div class="my-3 flex flex-row justify-start medium rounded-md h-3 w-1/3">
          
        </div>

        <!-- Build number -->
        <div class="my-3 flex flex-row justify-start medium rounded-md h-3 w-1/4">
          
        </div>

      </div>
      <div class="mt-2 mb-0 pt-2 text-right">
        <span class="medium rounded-md h-3 w-1/3 float-right"></span>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  props: ["build"],
  components: {
    
  },
  methods: {

  }
};
</script>

<style scoped>
  .light {
    @apply bg-gray-200;
  }

  .medium {
    @apply bg-gray-400;
  }

  .dark {
    @apply bg-gray-500;
  }
</style>