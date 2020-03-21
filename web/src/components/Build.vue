<template>
  <div class="cursor-pointer text-gray-100 p-4 rounded-lg shadow-lg transition duration-500 ease-in-out transform hover:scale-105" :class="getBackgroundClass()" @click="navigateToBuild()">
    <div class="flex flex-col justify-between h-full">
      <!-- Project -->
      <div class="flex justify-between align-middle">
        <div class="text-xl font-bold">{{ build.project }}</div>
        <BuildIcon :build="build" class />
      </div>
      <div>
        <!-- Project -->
        <div class="my-3 flex flex-row justify-start">
          <span>
            <fa-icon class="font-semibold" icon="hashtag" fixed-width />
          </span>
          <div class="ml-2 leading-normal">{{ build.build }}</div>
        </div>

        <!-- Branch -->
        <div class="my-3 flex flex-row justify-start">
          <span>
            <fa-icon class="font-semibold" icon="code-branch" fixed-width />
          </span>
          <span class="ml-2 leading-normal">{{ build.branch }}</span>
        </div>

        <!-- Build number -->
        <div class="my-3 flex flex-row justify-start">
          <span>
            <StatusIcon :build="build" />
          </span>
          <span class="ml-2">{{ getBuildKind() }} {{ build.buildNumber }}</span>
        </div>

      </div>
      <div class="mt-2 mb-0 border-grey-100 border-t-2 pt-2 text-right">
        <span>{{ getStatusText() }}&nbsp;</span>
        <span class="font-semibold" v-if="build.status != 'Running'">{{ build.finished | moment("from", "now") }}</span>
        <span v-if="build.status == 'Running'">{{ build.started | moment("from", "now") }}</span>
      </div>
    </div>
  </div>
</template>

<script>
import { library } from "@fortawesome/fontawesome-svg-core";
import { faHashtag, faCodeBranch } from "@fortawesome/free-solid-svg-icons";
import BuildIcon from "./BuildIcon.vue";
import StatusIcon from "./StatusIcon.vue";

library.add(faHashtag, faCodeBranch);

export default {
  props: ["build"],
  components: {
    BuildIcon,
    StatusIcon
  },
  methods: {
    navigateToBuild: function() {
      window.open(this.build.url, "_blank");
    },
    getBackgroundClass: function() {
      return this.build.status.toLowerCase();
    },
    getBuildKind: function() {
      switch (this.build.provider) {
        case "OctopusDeploy":
          return "Deployment";
        default:
          return "Build";
      }
    },
    getStatusText: function() {
      switch (this.build.status) {
        case "Success":
          return "Succeeded";
        case "Running":
          return "Started";
        default:
          return this.build.status;
      }
    }
  }
};
</script>

<style scoped>
.success {
  background-color: #66bb6a;
}

.failed {
  background-color: #d84b4b;
}

.running {
  background-color: #29b6f6;
}

.canceled {
  background-color: #999999;
}

.queued {
  background-color: #999999;
}
</style>