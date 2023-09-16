datadir = get_option('prefix') / get_option('datadir')

application_id = 'io.unobserved.espansoGUI'

install_data(
    'linux/@0@.desktop'.format(application_id),
    install_dir: datadir / 'applications'
)

install_data(
    'linux/@0@.appdata.xml'.format(application_id),
    install_dir: datadir / 'metainfo'
)

install_data(
    'linux/@0@.svg'.format(application_id),
    install_dir: datadir / 'icons' / 'hicolor' / 'scalable' / 'apps'
)
