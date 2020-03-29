<template>
  <div>
    <p>
      <span class="font-bold">Version:</span>
      {{ getVersion() }}
    </p>
    <p v-if="server != null">
      <span class="font-bold">Server:</span>
      {{ server }}
    </p>
    <p v-if="started != null">
      <span class="font-bold">Started:</span>
      {{ started | moment("from", "now") }}
    </p>
  </div>
</template>

<script>
import { data } from "@/js/store.js";

export default {
  computed: {
    server() {
      if (data.server == null || data.server === '') {
        return null;
      }
      return data.server;
    },
    started() {
      if (data.info == null) {
        return null;
      }
      return data.info.started;
    }
  },
  methods: {
    getVersion: function() {
      return data.version;
    }
  }
};
</script>