<template>
  <b-card :bg-variant="getBackground(build)" text-variant="light" tag="article" style="box-shadow:0px 0px 5px 5px rgba(0,0,0,0.2)">
    <template v-slot:header>
      <h6 class="mb-0">{{ build.project }}</h6>
      <span>{{ build.build }}</span>
    </template>
    <b-card-text>
      <table>
        <tr>
          <td>
            <font-awesome-icon v-if="build.status == 'Success'" :icon="['fas', 'check-circle']" />
            <font-awesome-icon
              v-if="build.status == 'Failed'"
              :icon="['fas', 'exclamation-triangle']"
            />
            <font-awesome-icon v-if="build.status == 'Running'" :icon="['fas', 'tools']" />
          </td>
          <td style="padding-left:10px">Build {{ build.buildNumber }}</td>
        </tr>
        <tr>
          <td style="padding-left:3px">
            <font-awesome-icon :icon="['fas', 'code-branch']" />
          </td>
          <td style="padding-left:10px">{{ build.branch }}</td>
        </tr>
      </table>
    </b-card-text>

    <template v-slot:footer>
      <table style="width:100%;">
        <tr>
          <td>
            <span v-if="build.status != 'Running'" class="small">{{ getBuildStatus(build) }}</span>
            <span
              v-if="build.status != 'Running'"
              class="small"
            >{{ build.finished | moment("from", "now") }}</span>
            <table v-if="build.status == 'Running'">
              <tr>
                <td>
                  <pulse-loader color="white" size="8px" class="span" />
                </td>
                <td style="padding-left:5px;">
                  <span class="small">{{ build.started | moment("from", "now") }}</span>
                </td>
              </tr>
            </table>
          </td>
          <td style="width:auto;text-align:right;white-space: nowrap">
            <a :href="build.url" target="_blank" title="Go to build">
              <img
                v-if="build.provider == 'TeamCity'"
                src="../assets/teamcity.svg"
                class="build-provider-avatar"
              />
              <img
                v-if="build.provider == 'AzureDevOps'"
                src="../assets/azuredevops.svg"
                class="build-provider-avatar"
              />
              <img
                v-if="build.provider == 'OctopusDeploy'"
                src="../assets/octopus.svg"
                class="build-provider-avatar"
              />
            </a>
          </td>
        </tr>
      </table>
    </template>
  </b-card>
</template>

<script>
import PulseLoader from "vue-spinner/src/PulseLoader.vue";

export default {
  props: ["build"],
  components: {
    PulseLoader
  },
  data() {
    return {};
  },
  methods: {
    getBuildStatus: function(build) {
      if (build.status == "Success") {
        return "Succeeded";
      }
      return build.status;
    },
    getBackground: function(build) {
      if (build.status == "Success") {
        return "success";
      } else if (build.status == "Failed") {
        return "danger";
      }
      return "info";
    }
  }
};
</script>

<style scoped>
.build-provider-avatar {
  opacity: 1;
  padding: 3px;
  border: 1px;
  border-style: solid;
  border-color: gray;
  border-radius: 5px;
  width: 32px;
  height: 32px;
  background: #fdfdfd;
  box-shadow:0px 0px 3px 3px rgba(0,0,0,0.15);
}
</style>