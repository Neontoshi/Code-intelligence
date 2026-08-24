// tests/fixtures/adversarial/dart/flutter_widget.dart

abstract class Widget {}
abstract class State<T> {}

class BuildContext {}

// Flutter Widget: lifecycle methods called dynamically by the Flutter framework
class UserCard extends Widget {
  final String userId;
  UserCard({required this.userId});

  // Framework lifecycle override: must not be flagged dead
  Widget build(BuildContext context) {
    return _renderContent();
  }

  Widget _renderContent() {
    return this;
  }

  // Dead function
  void _uncalledDartHelper() {
    // dead
  }
}

class UserCardState extends State<UserCard> {
  // Lifecycle methods called by engine
  void initState() {}
  void dispose() {}
}
