document.addEventListener('DOMContentLoaded', function () {
    let pairs = [
        ['a', 'href'],
        ['link', 'href'],
        ['script', 'src'],
        ['img', 'src'],
        ['source', 'src'],
        ['video', 'src'],
        ['audio', 'src']
        ['iframe', 'src']
    ]


    pairs.forEach(([tag, attr]) => {
        document.querySelectorAll(tag).forEach(element => {
            let link = element.getAttribute(attr);
            if (link && link.startsWith('/')) {
                element.setAttribute(attr, "{baseUrl}" + link.substring(1));
            }
        });
    });
});