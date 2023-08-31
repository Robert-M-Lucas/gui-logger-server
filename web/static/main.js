let id_counter = 0;
const body_element = document.getElementById("body");

function show_error(error) {
    body_element.innerHTML +=
`
<div class="d-flex align-items-center justify-content-center" style="position: absolute; left: 0; top: 0; width: 100vw; height: 100vh; background-color: rgba(255, 0, 0, 0.5)">
    <div><h1>${error}</h1></div>
</div>
`;
}

class Tree {
    constructor(parent, name) {
        this.parent = parent;
        this.name = name;
        this.type = "tree";
        this.elements = [];

        console.log(`Adding to ${parent.id}`);
        this.parent.root_container.innerHTML +=
`
<div class="container tree-container" id="${id_counter}">
    <h3><b>${name}</b></h3>
</div>
`;

        this.root_container = document.getElementById(id_counter.toString());
        this.id = id_counter;
        id_counter += 1;
    }

    destroy() {
        document.getElementById(this.id.toString()).remove();
    }


    get_path(path) {
        const current_level = path[0];

        for (const el of this.elements) {
            if (el.name === current_level) {
                if (path.length === 1) {
                    return [this, el];
                }
                else {
                    return el.get_path(path.slice(1, path.length));
                }
            }
        }

        if (path.length === 1) {
            return [this, null];
        }

        const new_tree = new Tree(this, current_level);
        this.elements.push(new_tree);
        return new_tree.get_path(path.slice(1, path.length));
    }
}

class ElementData {
    constructor(json) {
        const obj = JSON.parse(json);
        this.path = obj.path.split("/");
        this.type = obj.type;
        this.data = obj.data;
    }

    create(parent) {
        if (this.type === "text") {
            parent.elements.push(new TextElement(parent, this.path.at(-1), this.data));
        }
        else if (this.type === "progress") {
            parent.elements.push(new ProgressElement(parent, this.path.at(-1), this.data));
        }
        else {
            console.error(`Unknown type ${this.type}`);
        }
    }
}

const root_container = document.getElementById("root-container");
const root_tree = new Tree({ root_container: root_container}, "Debugger");


function update_element(element_data) {
    const result = root_tree.get_path(element_data.path);
    if (element_data.type === "delete") {
        if (result[1] != null) {
            console.log("Deleting");
            const element = result[1]
            const parent = element.parent;
            for (let i = 0;  i < parent.elements.length; i++) {
                if (parent.elements[i].name === element.name) {
                    parent.elements.splice(i, 1);
                    break;
                }
            }
            element.destroy();
        }
        else {
            console.error("Tried to delete non-existent element: " + element_data.path.join("/"));
        }
        return;
    }

    if (result[1] != null) { // Element exists
        const element = result[1];
        if (element.type === element_data.type) {
            // console.log("Element '" + element_data.path.join("/") + "' exists - updating...");
            element.update(element_data);
        }
        else {
            const parent = element.parent;
            for (let i = 0;  i < parent.elements.length; i++) {
                if (parent.elements[i].name === element.name) {
                    parent.elements.splice(i, 1);
                    break;
                }
            }
            element.destroy();
            element_data.create(parent);
        }
    }
    else {
        const parent = result[0];
        element_data.create(parent);
    }
}

const xmlHttp = new XMLHttpRequest();
xmlHttp.onreadystatechange = function() {
    if (xmlHttp.readyState === 4 && xmlHttp.status === 200) {
        console.log("Opening socket connection on port:" + xmlHttp.responseText);
        const socket = new WebSocket("ws://0.0.0.0:" + xmlHttp.responseText + "/ws");

        socket.addEventListener("message", (event) => {
            console.log(`Received data: ${event}`);
            update_element(new ElementData(event.data));
        });

        socket.addEventListener("close", (event) => {
            console.log(event);
            show_error("Web socket connection failed - reload page to retry");
        });
    }
    else if (xmlHttp.readyState === 4) {
        console.log(xmlHttp.responseText);
        show_error("Fetching web socket port failed - reload page to retry");
    }
}
xmlHttp.open("GET", "http://0.0.0.0:8000/port", true); // true for asynchronous
xmlHttp.send(null);

