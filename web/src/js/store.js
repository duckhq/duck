import Vue from "vue";
import axios from "axios";

//This represent the application state
export const data = Vue.observable({
    server: null,
    version: process.env.VUE_APP_VERSION,
    builds: null,
    view: null,
    info: null,
    error: false
});

export const store = {
    update(progress, server, view) {
        progress.start();

        if(server == undefined) {
            server = "";
        }

        data.server = server;

        let address = `${server}/api/builds`;
        if (view != undefined && view != null) {
            data.view = view;
            address = address + "/view/" + view;
        } else {
            data.view = null;
        }

        // Get all builds from the Duck server.
        axios
            .get(address)
            .then(response => {
                data.builds = response.data;
                data.error = false;

                progress.finish();

                if (data.info == null) {
                    // Get server information.
                    // We only need to do this once.
                    axios
                        .get(`${data.server}/api/server`)
                        .then(response => {
                            data.info = response.data;
                        })
                        .catch(() => {
                            data.info = null;
                        });
                }
            })
            .catch(() => {
                // Reset everything
                data.builds = null;
                data.error = true;
                data.info = null;
                progress.fail();
            })
            .finally(() => {
                data.loading = false;
            });
    },
};