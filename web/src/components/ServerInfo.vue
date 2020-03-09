<template>
  <div>
    <p>
      <b>Version:</b>
      {{ getVersion() }}
    </p>
    <p v-if="server != null">
      <b>Server:</b>
      {{ server }}
    </p>
    <p v-if="started != null">
      <b>Started:</b>
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