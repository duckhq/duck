<template>
  <img :src="getImagePath()" class="build-icon" />
</template>

<script>
export default {
  props: ["build"],
  methods: {
    getImagePath: function() {
      var images = require.context("../assets/", false, /\.svg$/);
      return images("./" + this.build.provider.toLowerCase() + ".svg");
    }
  }
};
</script>

<style scoped>
.build-icon {
  opacity: 1 !important;
  padding: 3px !important;
  border: 1px !important;
  border-style: solid !important;
  border-color: #999999 !important;
  border-radius: 5px !important;
  width: 32px !important;
  height: 32px !important;
  background: #f1f1f1 !important;
  box-shadow: 2px 2px 2px 2px rgba(0, 0, 0, 0.15) !important;
}
</style>