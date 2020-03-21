<template>
  <div class="mx-5 mt-5 text-red-800 border-red-800 border bg-red-100 p-5 rounded-lg flex flex-col text-center sm:text-left sm:flex-row sm:justify-left">
    <div class="mb-2 sm:mb-0 mx-auto sm:mx-0">
      <fa-icon icon="spinner" spin size="4x" />
    </div>
    <div class="ml-5">
      <span class="text-2xl font-bold">
        Could not connect to Duck server
      </span>
      <div class="mt-3">
        <div>{{ this.getMessage() }}</div>
        <div class="mt-2">Trying to reconnect...</div>
      </div>
    </div>
  </div>
</template>

<script>
import { library } from '@fortawesome/fontawesome-svg-core'
import { faSpinner } from '@fortawesome/free-solid-svg-icons'
import { data } from "@/js/store.js";

library.add(faSpinner);

export default {
  methods: {
    getMessage: function() {
      if (data.server === '') {
        return "The local Duck server could not be reached.";
      } else {
        return `The Duck server could not be reached at "${data.server}".`;
      }
    }
  }
};
</script>