async function handleClick() {
    console.log('click');
}

function addButton() {
    const button = document.createElement("button");
    button.id = "delete";
    //    button.classList.add("bg-blue-500", "hover:bg-blue-700", "text-white", "font-bold", "py-2", "px-4", "rounded");
    button.style.backgroundColor = 'rgb(255, 0, 0)';
    button.style.color = 'rgb(255, 255, 255)';
    button.style.padding = '0.5rem 1rem 0.5rem 1rem';
    button.style['border-radius'] = '0.25rem';
    button.style.margin = '40px';
    button.innerHTML = "Delete conversation";
    button.addEventListener("click", async function () {
        handleClick(button);
    });
    const buttonsWrapper = document.querySelector("#app");
    buttonsWrapper.appendChild(button);
}

function ensureButton() {
    if (!document.querySelector('#delete')) {
        addButton();
    }
}

setInterval(() => {
    ensureButton();
}, 500);
