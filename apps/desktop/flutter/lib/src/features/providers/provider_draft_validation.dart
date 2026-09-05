import 'package:linguaray_application/linguaray_application.dart';

enum ProviderDraftValidationIssue {
  missingId,
  unknownType,
  missingRequiredField,
}

final class ProviderDraftValidation {
  const ProviderDraftValidation.valid() : issue = null, fieldKey = null;

  const ProviderDraftValidation.invalid(this.issue, {this.fieldKey});

  final ProviderDraftValidationIssue? issue;
  final String? fieldKey;

  bool get isValid => issue == null;
  String? get errorCode => isValid ? null : 'validation_missing';
}

ProviderDraftValidation validateProviderDraft({
  required ProviderDraft draft,
  required ProviderTypeOption? type,
  Set<String> storedSecretKeys = const {},
  Set<String> ignoredRequiredFields = const {},
}) {
  if (draft.id.trim().isEmpty) {
    return const ProviderDraftValidation.invalid(
      ProviderDraftValidationIssue.missingId,
    );
  }
  if (type == null) {
    return const ProviderDraftValidation.invalid(
      ProviderDraftValidationIssue.unknownType,
    );
  }
  for (final field in type.fields) {
    if (!field.requiredField || ignoredRequiredFields.contains(field.key)) {
      continue;
    }
    final value = draft.fields[field.key]?.trim() ?? '';
    final keepsStoredSecret =
        field.secret && storedSecretKeys.contains(field.key);
    if (value.isEmpty && !keepsStoredSecret) {
      return ProviderDraftValidation.invalid(
        ProviderDraftValidationIssue.missingRequiredField,
        fieldKey: field.key,
      );
    }
  }
  return const ProviderDraftValidation.valid();
}
