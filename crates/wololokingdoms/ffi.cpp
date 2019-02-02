#include <map>
#include <wksettings.h>
#include <wkconverter.h>

typedef void* wkconverter_t;

struct wksettings_t {
  bool use_voobly;
  bool use_exe;
  bool use_both;
  bool use_monks;
  bool use_pw;
  bool use_walls;
  bool copy_maps;
  bool copy_custom_maps;
  bool restricted_civ_mods;
  bool use_no_snow;
  bool fix_flags;
  bool replace_tooltips;
  bool use_grid;
  char* install_directory;
  char* language;
  int dlc_level;
  int patch;
  int hotkey_choice;
  char* hd_path;
  char* out_path;
  char* voobly_dir;
  char* up_dir;
  char* mod_name;
};

struct wklistener_t {
  void* data;
  void (*finished) (void*);
  void (*log) (void*, char*);
  void (*set_info) (void*, char*);
  void (*error) (void*, char*);
  void (*create_dialog) (void*, char*);
  void (*create_dialog_title) (void*, char*, char*);
  void (*create_dialog_replace) (void*, char*, char*, char*);
  void (*set_progress) (void*, int);
  void (*install_userpatch) (void*, char*, char**);
};

class FFIConvertListener: public WKConvertListener {
  wklistener_t* listener;
public:
  FFIConvertListener (wklistener_t* listener) : listener(listener) {}
  virtual void finished () {
    if (listener->finished) {
      listener->finished(listener->data);
    }
  }
  virtual void log(std::string msg) {
    if (listener->log) {
      listener->log(listener->data, msg.c_str());
    }
  }
  virtual void setInfo(std::string msg) {
    if (listener->set_info) {
      listener->set_info(listener->data, msg.c_str());
    }
  }
  virtual void error(std::exception const& err) {
    if (listener->error) {
      listener->error(listener->data, err.what().c_str());
    }
  }
  virtual void setProgress(int i) {
    if (listener->set_progress) {
      listener->set_progress(listener->data, i);
    }
  }
  virtual void installUserPatch(fs::path userPatchExe, std::vector<std::string> cliFlags) {
    if (!listener->install_userpatch) {
      return;
    }

    auto num_flags = cliFlags.size();
    auto flags = new char*[num_flags + 1];
    auto i = 0;
    for (auto& f : cliFlags) {
      flags[i++] = f.c_str();
    }
    listener->install_userpatch(listener->data, userPatchExe.string().c_str(), flags);
  }
};

extern "C" wkconverter_t wkconverter_create (wksettings_t* settings, wklistener_t* listener) {
  auto settings = new WKSettings(
    settings->use_voobly,
    settings->use_exe,
    settings->use_both,
    settings->use_monks,
    settings->use_pw,
    settings->use_walls,
    settings->copy_maps,
    settings->copy_custom_maps,
    settings->restricted_civ_mods,
    settings->use_no_snow,
    settings->fix_flags,
    settings->replace_tooltips,
    settings->use_grid,
    settings->install_directory,
    settings->language,
    settings->dlc_level,
    settings->patch,
    settings->hotkey_choice,
    settings->hd_path,
    settings->out_path,
    settings->voobly_dir,
    settings->up_dir,
    std::map(),
    settings->mod_name
  );
  auto convert_listener = new FFIConvertListener(listener);

  auto converter = new WKConverter(settings, convert_listener);

  return converter;
}

extern "C" void wkconverter_run (wkconverter_t converter) {
  auto wkc = reinterpret_cast<WKConverter*>(converter);
  wkc->run();
}
