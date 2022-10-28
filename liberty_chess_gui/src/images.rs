use usvg::Tree;

fn load_image(data: &[u8]) -> Tree {
  Tree::from_data(data, &usvg::Options::default().to_ref()).unwrap()
}

pub fn get_images() -> [Tree; 36] {
  [
    load_image(include_bytes!("../../resources/WPawn.svg")),
    load_image(include_bytes!("../../resources/WKnight.svg")),
    load_image(include_bytes!("../../resources/WBishop.svg")),
    load_image(include_bytes!("../../resources/WRook.svg")),
    load_image(include_bytes!("../../resources/WQueen.svg")),
    load_image(include_bytes!("../../resources/WKing.svg")),
    load_image(include_bytes!("../../resources/WArchbishop.svg")),
    load_image(include_bytes!("../../resources/WChancellor.svg")),
    load_image(include_bytes!("../../resources/WCamel.svg")),
    load_image(include_bytes!("../../resources/WZebra.svg")),
    load_image(include_bytes!("../../resources/WMann.svg")),
    load_image(include_bytes!("../../resources/WNightrider.svg")),
    load_image(include_bytes!("../../resources/WChampion.svg")),
    load_image(include_bytes!("../../resources/WCentaur.svg")),
    load_image(include_bytes!("../../resources/WAmazon.svg")),
    load_image(include_bytes!("../../resources/WElephant.svg")),
    load_image(include_bytes!("../../resources/WObstacle.svg")),
    load_image(include_bytes!("../../resources/WWall.svg")),
    load_image(include_bytes!("../../resources/BPawn.svg")),
    load_image(include_bytes!("../../resources/BKnight.svg")),
    load_image(include_bytes!("../../resources/BBishop.svg")),
    load_image(include_bytes!("../../resources/BRook.svg")),
    load_image(include_bytes!("../../resources/BQueen.svg")),
    load_image(include_bytes!("../../resources/BKing.svg")),
    load_image(include_bytes!("../../resources/BArchbishop.svg")),
    load_image(include_bytes!("../../resources/BChancellor.svg")),
    load_image(include_bytes!("../../resources/BCamel.svg")),
    load_image(include_bytes!("../../resources/BZebra.svg")),
    load_image(include_bytes!("../../resources/BMann.svg")),
    load_image(include_bytes!("../../resources/BNightrider.svg")),
    load_image(include_bytes!("../../resources/BChampion.svg")),
    load_image(include_bytes!("../../resources/BCentaur.svg")),
    load_image(include_bytes!("../../resources/BAmazon.svg")),
    load_image(include_bytes!("../../resources/BElephant.svg")),
    load_image(include_bytes!("../../resources/BObstacle.svg")),
    load_image(include_bytes!("../../resources/BWall.svg")),
  ]
}
