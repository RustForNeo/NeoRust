const neo = import("./pkg");

neo
    .then(m => {
        m.deploy().catch(console.error);
    })
    .catch(console.error);
