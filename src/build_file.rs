


pub fn build_bolt_file_basic(name:&str,description:&str) -> String{

    let beet_json:&str  = r#"
    {
 "name":"NAME",
  "description":"DESCRIPTION",

    "require": [
        "bolt"
    ],

    "data_pack":{
        "load": ["src"]
    },
    
    "pipeline": [
        "mecha"
    ],
    


    "output": "build",

    "meta":{
                "bolt":{
            "entrypoint":["NAME:main"]
        }
    }
}

    
    "#;

    let beet_json: String =beet_json.replace("NAME", name).replace("DESCRIPTION", description);
    return beet_json;

}


pub fn build_bolt_file_resourcepack(name:&str,description:&str) -> String{
    let beet_json:&str  = r#"
    {
  "name": "NAME",
  "description": "DESCRIPTION",
  "require": [
    "bolt"
  ],
  "data_pack": {
    "load": [
      "src"
    ]
  },
  "resource_pack": {
    "load": [
      "src"
    ]
  },
  "pipeline": [
    "mecha"
  ],
  "output": "build",
  "meta": {
    "bolt": {
      "entrypoint": [
        "NAME:main"
      ]
    }
  }
}

    
    "#;

    let beet_json: String =beet_json.replace("NAME", name).replace("DESCRIPTION", description);
    return beet_json;
}


pub fn build_bolt_file_version(name:&str,description:&str,version:&str) -> String{
    let beet_json:&str  = r#"
    {
  "name": "NAME",
  "description": "DESCRIPTION",
  "require": [
    "bolt"
  ],
  "data_pack": {
    "load": [
      "src"
    ],
    "pack_format": VERSION
  },
  "pipeline": [
    "mecha"
  ],
  "output": "build",
  "meta": {
    "bolt": {
      "entrypoint": [
        "NAME:main"
      ]
    }
  }
}

    
    "#;

    let beet_json: String =beet_json.replace("NAME", name).replace("DESCRIPTION", description).replace("VERSION", version);
    return beet_json;
}


pub fn build_bolt_file_all(name:&str,description:&str,version:&str) -> String{
    let beet_json:&str  = r#"
    {
  "name": "NAME",
  "description": "DESCRIPTION",
  "require": [
    "bolt"
  ],
  "data_pack": {
    "load": [
      "src"
    ],
    "pack_format": VERSION
  },
  "resource_pack": {
    "load": [
      "src"
    ],
    "pack_format": VERSION
  },
  "pipeline": [
    "mecha"
  ],
  "output": "build",
  "meta": {
    "bolt": {
      "entrypoint": [
        "NAME:main"
      ]
    }
  }
}

    
    "#;

    let beet_json: String =beet_json.replace("NAME", name).replace("DESCRIPTION", description).replace("VERSION", version);
    return beet_json;
}