<template>
  <div class="ui icon big negative message">
    <i class="spinner circle loading icon"></i>
    <div class="content">
      <div class="header">Could not connect to Duck server</div>
      <p>
        <span>{{ this.getMessage() }}</span>
        <br />
        <span>Trying to reconnect...</span>
      </p>
    </div>
  </div>
</template>

<script>
import { data } from "@/js/store.js";

export default {
  methods: {
    getMessage: function() {
      if (data.server === '') {
        return "The local Duck server could not be reached.";
      } else {
        return `The Duck server could not be reached at ${data.server}.`;
      }
    }
  }
};
</script>