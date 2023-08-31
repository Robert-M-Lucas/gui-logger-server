class TextElement {
    constructor(parent, name, data) {
        this.parent = parent;
        this.name = name;
        this.text = data
        this.type = "text";

        parent.root_container.innerHTML +=
            `<div class="card mb-2" id="${id_counter}"><p class="m-1 p-0">${this.name}: ${this.text}</p></div>`;

        this.root_container = document.getElementById(id_counter.toString());
        this.id = id_counter;
        id_counter += 1;
    }


    update(element_data) {
        this.text = element_data.data;
        // console.log("Updating '" + this.name + "' to '" + this.text + "'");
        // console.log(this.root_container);
        document.getElementById(this.id.toString()).innerHTML = `<p class="m-1 p-0">${this.name}: ${this.text}</p>`;
    }

    destroy() {
        document.getElementById(this.id.toString()).remove();
    }
}

class ProgressElement {
    constructor(parent, name, data) {
        this.parent = parent;
        this.name = name;
        this.progress = data
        this.type = "progress";

        parent.root_container.innerHTML +=
`
<div class="card mb-2" id="${id_counter}">
    <div class="card-title m-1">${this.name}</div>
    <div class="progress m-1">
        <div class="progress-bar bg-success" id="${id_counter + 1}" role="progressbar" aria-label="Success example" style="width: ${this.progress}%" aria-valuenow="0.1" aria-valuemin="0" aria-valuemax="100">${this.progress}%</div>
    </div>
</div>
`;

        this.root_container = document.getElementById(id_counter.toString());
        this.id = id_counter;
        this.progress_id = id_counter + 1;
        id_counter += 2;
    }


    update(element_data) {
        this.progress = element_data.data;
        let el = document.getElementById(this.progress_id.toString());
        el.style.width = this.progress + "%";
        el.innerHTML = this.progress + "%";
    }

    destroy() {
        document.getElementById(this.id.toString()).remove();
    }
}