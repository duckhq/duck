<template>
  <img :src="getImagePath()" class="w-8 h-8 border border-gray-200 p-1 rounded-sm shadow-md bg-gray-100" />
</template>

<script>
import { imageLoader } from "@/js/config";

export default {
  props: ["build"],
  methods: {
    getImagePath: function() {
      return imageLoader("./" + this.build.provider.toLowerCase() + ".svg");
    }
  }
};
</script>

<style scoped>

</style>