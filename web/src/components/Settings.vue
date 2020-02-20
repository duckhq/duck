<template>
  <transition name="modal">
    <div class="ui modal-mask">
      <div class="modal-wrapper">
        <div class="modal-container">
          <div class="ui raised segment">
            <!-- Header -->
            <a class="ui blue ribbon label" style="margin-left: -1px;">
              <i class="cog icon"></i>Settings
            </a>
            <!-- Version -->
            <div class="ui top right attached label">
              <small>
                <a href="https://github.com/spectresystems/duck" target="_blank">Duck v{{ getVersion() }}</a>
              </small>
            </div>
            <!-- Body -->
            <div class="modal-body">
              <div class="ui top attached tabular menu">
                <a class="active item">Views</a>
              </div>
              <div class="ui bottom attached segment">
                <!-- Views -->
                <ViewList @view_changed="$emit('close')" />
              </div>
            </div>
            <!-- Footer -->
            <div class="modal-footer">
              <button class="ui button right floated" @click="$emit('close')">Close</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<script>
import ViewList from "./ViewList.vue";
import { data } from "@/js/store.js";

export default {
  props: ["currentView", "views"],
  components: {
    ViewList
  },
  methods: {
    getVersion() {
      return data.version;
    }
  }
};
</script>

<style scoped>
/* Container */
.modal-mask {
  position: fixed;
  z-index: 99999;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.6);
  display: table;
  transition: opacity 0.3s ease;
}
.modal-wrapper {
  display: table-cell;
  vertical-align: middle;
}
.modal-container {
  width: 450px;
  margin: 0px auto;
  padding: 20px 25px 25px 25px;
  background-color: transparent;
  transition: all 0.3s ease;
  font-family: Helvetica, Arial, sans-serif;
  overflow-y: auto;
}
.modal-container .label {
  opacity: 0.9;
}

/* Content */
.modal-body {
  padding: 20px 0px 20px 0px;
}
.modal-body .label {
  opacity: 0.6;
}
.modal-body .label a {
  text-decoration: none;
  color: black;
  opacity: 0.6;
}
.modal-footer {
  padding-bottom: 40px;
}

/* Transitions */
.modal-leave-active {
  opacity: 0;
  position: absolute;
}
.modal-leave-active .modal-container {
  opacity: 0;
  transform: scale(2.1);
}
</style>