<template>
  <div>
    <p v-if="version != null">
      <span class="font-bold">Version:</span>
      <span id="version">{{ version }}</span>
    </p>
    <p v-if="server != null">
      <span class="font-bold">Server:</span>
      <span id="server">{{ server }}</span>
    </p>
    <p v-if="started != null">
      <span class="font-bold">Started:</span>
      <span id="started">{{ started | moment("from", "now") }}</span>
    </p>
  </div>
</template>

<script>
export default {
  props: {
    data: {}
  },

  computed: {
    server() {
      if (this.data?.server == null || this.data?.server === '') {
        return null;
      }
      return this.data.server;
    },
    started() {
      if (this.data?.info == null) {
        return null;
      }
      return this.data.info.started;
    },
    version: function() {
      if (this.data == null) {
        return null;
      }
      return this.data.version;
    }
  }
};
</script>