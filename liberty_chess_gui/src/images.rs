use usvg::Tree;

fn load_image(data: &[u8]) -> Tree {
  Tree::from_data(data, &usvg::Options::default().to_ref()).unwrap()
}

pub fn get() -> [Tree; 36] {
  [
    load_image(include_bytes!("../../resources/images/WPawn.svg")),
    load_image(include_bytes!("../../resources/images/WKnight.svg")),
    load_image(include_bytes!("../../resources/images/WBishop.svg")),
    load_image(include_bytes!("../../resources/images/WRook.svg")),
    load_image(include_bytes!("../../resources/images/WQueen.svg")),
    load_image(include_bytes!("../../resources/images/WKing.svg")),
    load_image(include_bytes!("../../resources/images/WArchbishop.svg")),
    load_image(include_bytes!("../../resources/images/WChancellor.svg")),
    load_image(include_bytes!("../../resources/images/WCamel.svg")),
    load_image(include_bytes!("../../resources/images/WZebra.svg")),
    load_image(include_bytes!("../../resources/images/WMann.svg")),
    load_image(include_bytes!("../../resources/images/WNightrider.svg")),
    load_image(include_bytes!("../../resources/images/WChampion.svg")),
    load_image(include_bytes!("../../resources/images/WCentaur.svg")),
    load_image(include_bytes!("../../resources/images/WAmazon.svg")),
    load_image(include_bytes!("../../resources/images/WElephant.svg")),
    load_image(include_bytes!("../../resources/images/WObstacle.svg")),
    load_image(include_bytes!("../../resources/images/WWall.svg")),
    load_image(include_bytes!("../../resources/images/BPawn.svg")),
    load_image(include_bytes!("../../resources/images/BKnight.svg")),
    load_image(include_bytes!("../../resources/images/BBishop.svg")),
    load_image(include_bytes!("../../resources/images/BRook.svg")),
    load_image(include_bytes!("../../resources/images/BQueen.svg")),
    load_image(include_bytes!("../../resources/images/BKing.svg")),
    load_image(include_bytes!("../../resources/images/BArchbishop.svg")),
    load_image(include_bytes!("../../resources/images/BChancellor.svg")),
    load_image(include_bytes!("../../resources/images/BCamel.svg")),
    load_image(include_bytes!("../../resources/images/BZebra.svg")),
    load_image(include_bytes!("../../resources/images/BMann.svg")),
    load_image(include_bytes!("../../resources/images/BNightrider.svg")),
    load_image(include_bytes!("../../resources/images/BChampion.svg")),
    load_image(include_bytes!("../../resources/images/BCentaur.svg")),
    load_image(include_bytes!("../../resources/images/BAmazon.svg")),
    load_image(include_bytes!("../../resources/images/BElephant.svg")),
    load_image(include_bytes!("../../resources/images/BObstacle.svg")),
    load_image(include_bytes!("../../resources/images/BWall.svg")),
  ]
}
