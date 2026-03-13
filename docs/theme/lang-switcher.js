// Language switcher for multi-language mdBook setup
(function() {
    var languages = [
        { code: '',       label: 'English' },
        { code: 'ru',     label: 'Русский' },
        { code: 'uk',     label: 'Українська' },
        { code: 'rs',     label: 'Srpski' },
        { code: 'rs-cyr', label: 'Српски' }
    ];

    var path = window.location.pathname;

    // Detect base path (handles /ibkr-porez-rs/ or /ibkr-porez/ prefix)
    var baseMatch = path.match(/^(\/[^/]+-porez(?:-rs)?\/)/);
    var basePath = baseMatch ? baseMatch[1] : '/';

    // Detect current language from path
    var currentLang = '';
    for (var i = 1; i < languages.length; i++) {
        if (path.indexOf(basePath + languages[i].code + '/') === 0) {
            currentLang = languages[i].code;
            break;
        }
    }

    var switcher = document.createElement('div');
    switcher.className = 'lang-switcher';

    languages.forEach(function(lang) {
        var a = document.createElement('a');
        a.href = basePath + (lang.code ? lang.code + '/' : '');
        a.textContent = lang.label;
        if (lang.code === currentLang) {
            a.className = 'active';
        }
        switcher.appendChild(a);
    });

    // Insert at the top of the content area
    var content = document.getElementById('content');
    if (content) {
        content.insertBefore(switcher, content.firstChild);
    }
})();
