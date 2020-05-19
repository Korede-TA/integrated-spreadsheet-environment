enum Action {
    ..
   ChangeEvent(Coordinate, ChangeEventMsg),
 }
 
 enum ChangeEventMsg {
   ChangeInput,
   InsertRow,
   InsertCol,
   DeleteCol,
   DeleteRow,
   AddNestedTable,
 }
 struct Model {
    observers: HashMap<Coordinate, Vec<Coordinate>>,
  }
  
 impl Component for Model {
 
 fn udpate(&mut self, action: Action) {
 match action {
   Action::ChangeEvent(current_coord, msg) {
     // blah blah
     for observer_coord in self.observers.get(current_coord).unwrap() {
       self.update(observer_coord)
     }
     true
   }
 }
 }